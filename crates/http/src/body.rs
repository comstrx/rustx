use http_body::{Body as HttpBody, Frame, SizeHint};
use core::task::{Context, Poll};
use error::{Failure, Result};
use http_body_util::BodyExt;
use bytes::Bytes as Shared;
use hyper::body::Incoming;
use core::mem::take;
use core::pin::Pin;
use memory::Bytes;

use crate::arch::{Flow, Polled};
use crate::{Body, Faults};

impl Body {

    /// No bytes.
    #[inline]
    #[must_use]
    pub const fn empty () -> Self {

        Self { flow: Flow::Empty }

    }

    /// Every byte in hand.
    #[inline]
    #[must_use]
    pub fn full ( bytes: impl Into<Bytes> ) -> Self {

        Self { flow: Flow::Full(Shared::from(bytes.into())) }

    }

    /// The same body refused past `limit` bytes; bytes already in hand are not re-checked.
    #[inline]
    #[must_use]
    pub fn limited ( self, limit: usize ) -> Self {

        match self.flow {
            Flow::Incoming(incoming) | Flow::Limited(incoming, _) => Self { flow: Flow::Limited(incoming, limit) },
            flow @ ( Flow::Empty | Flow::Full(_) )               => Self { flow },
        }

    }

    /// How many bytes are known to be coming, when that is known.
    #[inline]
    #[must_use]
    pub fn len ( &self ) -> Option<u64> {

        match &self.flow {
            Flow::Empty                                           => Some(0),
            Flow::Full(bytes)                                     => Some(u64::try_from(bytes.len()).unwrap_or(u64::MAX)),
            Flow::Incoming(incoming) | Flow::Limited(incoming, _) => incoming.size_hint().exact(),
        }

    }

    /// Whether no byte will come.
    #[inline]
    #[must_use]
    pub fn is_empty ( &self ) -> bool {

        self.len() == Some(0)

    }

    /// Every byte, gathered; `TooLarge` keyed `body_too_large` past `limit`.
    ///
    /// # Errors
    /// The peer went away, or the body passed the limit.
    #[inline]
    pub async fn gather ( self, limit: usize ) -> Result<Bytes> {

        match self.flow {
            Flow::Empty                  => Ok(Bytes::new()),
            Flow::Full(bytes)            => if bytes.len() <= limit { Ok(Bytes::from(bytes)) } else { Err(Self::too_large()) },
            Flow::Incoming(incoming)     => Self::drain(incoming, limit).await,
            Flow::Limited(incoming, cap) => Self::drain(incoming, cap.min(limit)).await,
        }

    }

    pub(crate) const fn arriving ( incoming: Incoming ) -> Self {

        Self { flow: Flow::Incoming(incoming) }

    }

    async fn drain ( incoming: Incoming, limit: usize ) -> Result<Bytes> {

        if incoming.size_hint().lower() > u64::try_from(limit).unwrap_or(u64::MAX) { return Err(Self::too_large()); }

        let collected = BodyExt::collect(Self { flow: Flow::Limited(incoming, limit) }).await?;

        Ok(Bytes::from(collected.to_bytes()))

    }

    fn too_large () -> Failure {

        Faults::BODY_TOO_LARGE.fail()

    }

    fn carried ( error: hyper::Error ) -> Failure {

        Faults::BODY_INTERRUPTED.fail().caused_by(error)

    }

    fn poll_limited ( incoming: &mut Incoming, left: &mut usize, context: &mut Context<'_> ) -> Polled {

        let Poll::Ready(next) = Pin::new(incoming).poll_frame(context) else { return Poll::Pending; };

        let Some(outcome) = next else { return Poll::Ready(None); };
        let frame = match outcome { Ok(frame) => frame, Err(error) => return Poll::Ready(Some(Err(Self::carried(error)))) };
        let size  = frame.data_ref().map_or(0, Shared::len);

        if size > *left { return Poll::Ready(Some(Err(Self::too_large()))); }

        *left -= size;

        Poll::Ready(Some(Ok(frame)))

    }

}

impl From<Bytes> for Body {

    #[inline]
    fn from ( value: Bytes ) -> Self {

        Self::full(value)

    }

}

impl From<&'static str> for Body {

    #[inline]
    fn from ( value: &'static str ) -> Self {

        Self::full(value)

    }

}

impl From<String> for Body {

    #[inline]
    fn from ( value: String ) -> Self {

        Self::full(value)

    }

}

impl From<Vec<u8>> for Body {

    #[inline]
    fn from ( value: Vec<u8> ) -> Self {

        Self::full(value)

    }

}

impl HttpBody for Body {

    type Data = Shared;
    type Error = Failure;

    #[inline]
    fn poll_frame ( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Option<Result<Frame<Shared>>>> {

        match &mut self.get_mut().flow {
            Flow::Empty => Poll::Ready(None),
            Flow::Full(bytes) => {
                let data = take(bytes);
                Poll::Ready((!data.is_empty()).then(|| Ok(Frame::data(data))))
            }
            Flow::Incoming(incoming) => Pin::new(incoming).poll_frame(cx).map(|frame| frame.map(|frame| frame.map_err(Self::carried))),
            Flow::Limited(incoming, left) => Self::poll_limited(incoming, left, cx),
        }

    }

    #[inline]
    fn is_end_stream ( &self ) -> bool {

        match &self.flow {
            Flow::Empty => true,
            Flow::Full(bytes) => bytes.is_empty(),
            Flow::Incoming(incoming) | Flow::Limited(incoming, _) => incoming.is_end_stream(),
        }

    }

    #[inline]
    fn size_hint ( &self ) -> SizeHint {

        match &self.flow {
            Flow::Empty => SizeHint::with_exact(0),
            Flow::Full(bytes) => SizeHint::with_exact(u64::try_from(bytes.len()).unwrap_or(u64::MAX)),
            Flow::Incoming(incoming) | Flow::Limited(incoming, _) => incoming.size_hint(),
        }

    }

}
