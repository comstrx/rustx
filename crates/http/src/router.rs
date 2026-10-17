use core::fmt::{Debug, Formatter, Result as FmtResult};
use hyper::header::HeaderValue;
use core::future::{Future, Ready, ready};
use error::Result;
use hyper::body::Incoming;
use std::time::Instant;
use core::cell::Cell;
use alloc::rc::Rc;
use hyper::Method;
use net::Peer;

use crate::{Body, Faults, Header, Ident, Middleware, Next, Params, Pending, Request, Response, Router};
use crate::arch::{Handler, Table};

impl Router {

    /// No routes, no middleware, `404` for everything, a megabyte of body.
    #[inline]
    #[must_use]
    pub fn new () -> Self {

        Self { routes: matchit::Router::new(), patterns: Vec::new(), tables: Vec::new(), layers: Vec::new(), missing: Self::boxed(Self::not_found), denied: Self::boxed(Self::not_allowed), limit: 1 << 20, worker: 0, serial: Cell::new(0) }

    }

    /// `GET` and, unless one is set, `HEAD`.
    #[inline]
    #[must_use]
    pub fn get <F, Fut> ( self, path: &str, handler: F ) -> Self
    where F: Fn(Request) -> Fut + 'static, Fut: Future<Output = Result<Response>> + 'static {

        self.add(path, |table, handler| table.get = Some(handler), handler)

    }

    /// `POST`.
    #[inline]
    #[must_use]
    pub fn post <F, Fut> ( self, path: &str, handler: F ) -> Self
    where F: Fn(Request) -> Fut + 'static, Fut: Future<Output = Result<Response>> + 'static {

        self.add(path, |table, handler| table.post = Some(handler), handler)

    }

    /// `PUT`.
    #[inline]
    #[must_use]
    pub fn put <F, Fut> ( self, path: &str, handler: F ) -> Self
    where F: Fn(Request) -> Fut + 'static, Fut: Future<Output = Result<Response>> + 'static {

        self.add(path, |table, handler| table.put = Some(handler), handler)

    }

    /// `PATCH`.
    #[inline]
    #[must_use]
    pub fn patch <F, Fut> ( self, path: &str, handler: F ) -> Self
    where F: Fn(Request) -> Fut + 'static, Fut: Future<Output = Result<Response>> + 'static {

        self.add(path, |table, handler| table.patch = Some(handler), handler)

    }

    /// `DELETE`.
    #[inline]
    #[must_use]
    pub fn delete <F, Fut> ( self, path: &str, handler: F ) -> Self
    where F: Fn(Request) -> Fut + 'static, Fut: Future<Output = Result<Response>> + 'static {

        self.add(path, |table, handler| table.delete = Some(handler), handler)

    }

    /// Every method the path has no handler for.
    #[inline]
    #[must_use]
    pub fn any <F, Fut> ( self, path: &str, handler: F ) -> Self
    where F: Fn(Request) -> Fut + 'static, Fut: Future<Output = Result<Response>> + 'static {

        self.add(path, |table, handler| table.any = Some(handler), handler)

    }

    /// Runs this middleware around every route, outermost first.
    #[inline]
    #[must_use]
    pub fn layer ( mut self, middleware: impl Middleware ) -> Self {

        self.layers.push(Rc::new(middleware));

        self

    }

    /// What answers when no path matches.
    #[inline]
    #[must_use]
    pub fn fallback <F, Fut> ( mut self, handler: F ) -> Self
    where F: Fn(Request) -> Fut + 'static, Fut: Future<Output = Result<Response>> + 'static {

        self.missing = Self::boxed(handler);

        self

    }

    /// How many body bytes a handler may gather unless it says otherwise.
    #[inline]
    #[must_use]
    pub const fn limit ( mut self, bytes: usize ) -> Self {

        self.limit = bytes;

        self

    }

    /// Which worker the ids this router issues belong to.
    #[inline]
    #[must_use]
    pub const fn worker ( mut self, worker: u16 ) -> Self {

        self.worker = worker;

        self

    }

    /// Routes one request through the middleware to its handler; a failure becomes its response.
    #[inline]
    pub async fn handle ( &self, request: Request ) -> Response {

        let ident   = request.ident;
        let outcome = self.route(request).await;

        let mut response = outcome.unwrap_or_else(Response::from);

        if let Ok(value) = HeaderValue::from_str(&ident.to_string()) { response.inner.headers_mut().insert(Header::REQUEST_ID.inner, value); }

        response

    }

    pub(crate) fn take ( &self, inner: hyper::Request<Incoming>, peer: Peer ) -> Request {

        let serial = self.serial.get().wrapping_add(1);

        self.serial.set(serial);

        Request { inner: inner.map(Body::arriving), peer, params: Params::default(), ident: Ident { worker: self.worker, serial }, started: Instant::now(), limit: self.limit }

    }

    async fn route ( &self, mut request: Request ) -> Result<Response> {

        let ( handler, params ) = self.pick(request.inner.method(), request.path());

        request.params = params;

        Next { layers: &self.layers, handler }.run(request).await

    }

    fn pick ( &self, method: &Method, path: &str ) -> ( &Handler, Params ) {

        let Ok(found) = self.routes.at(path) else { return ( &self.missing, Params::default() ); };
        let handler = self.tables.get(*found.value).and_then(|table| table.pick(method)).unwrap_or(&self.denied);

        ( handler, Params::capture(&found.params) )

    }

    fn add <F, Fut> ( mut self, path: &str, place: fn(&mut Table, Handler), handler: F ) -> Self
    where F: Fn(Request) -> Fut + 'static, Fut: Future<Output = Result<Response>> + 'static {

        let handler = Self::boxed(handler);
        let index   = self.patterns.iter().position(|pattern| &**pattern == path).unwrap_or_else(|| self.register(path));

        if let Some(table) = self.tables.get_mut(index) { place(table, handler); }

        self

    }

    fn register ( &mut self, path: &str ) -> usize {

        let index = self.tables.len();

        self.patterns.push(path.into());
        self.tables.push(Table::default());
        self.routes.insert(path, index).unwrap_or_default();

        index

    }

    fn boxed <F, Fut> ( handler: F ) -> Handler
    where F: Fn(Request) -> Fut + 'static, Fut: Future<Output = Result<Response>> + 'static {

        Rc::new(move |request| Box::pin(handler(request)))

    }

    fn not_allowed ( request: Request ) -> Ready<Result<Response>> {

        drop(request);

        ready(Err(Faults::METHOD_NOT_ALLOWED.fail()))

    }

    fn not_found ( request: Request ) -> Ready<Result<Response>> {

        let path = request.path().to_owned();

        drop(request);

        ready(Err(Faults::NO_ROUTE.fail().context(path)))

    }

}

impl Default for Router {

    #[inline]
    fn default () -> Self {

        Self::new()

    }

}

impl Debug for Router {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.debug_struct("Router").field("layers", &self.layers.len()).field("limit", &self.limit).field("worker", &self.worker).finish_non_exhaustive()

    }

}

impl Table {

    pub(crate) fn pick ( &self, method: &Method ) -> Option<&Handler> {

        let exact = match *method {
            Method::GET     => self.get.as_ref(),
            Method::POST    => self.post.as_ref(),
            Method::PUT     => self.put.as_ref(),
            Method::PATCH   => self.patch.as_ref(),
            Method::DELETE  => self.delete.as_ref(),
            Method::HEAD    => self.head.as_ref().or(self.get.as_ref()),
            Method::OPTIONS => self.options.as_ref(),
            _               => None,
        };

        exact.or(self.any.as_ref())

    }

}

impl Debug for Table {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.debug_struct("Table").field("get", &self.get.is_some()).field("post", &self.post.is_some()).field("any", &self.any.is_some()).finish_non_exhaustive()

    }

}

impl <'a> Next<'a> {

    /// Runs the rest of the chain on the request.
    #[inline]
    pub fn run ( self, request: Request ) -> Pending<'a> {

        match self.layers.split_first() {
            Some(( layer, rest )) => layer.call(request, Self { layers: rest, handler: self.handler }),
            None                  => (self.handler)(request),
        }

    }

}

impl Debug for Next<'_> {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.debug_struct("Next").field("layers", &self.layers.len()).finish_non_exhaustive()

    }

}
