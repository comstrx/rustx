use tokio::io::{AsyncRead, AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _, ReadBuf};
use std::io::{IoSlice, Result as IoResult};
use core::task::{Context, Poll};
use error::{Failure, Result};
use std::net::TcpStream as StdStream;
use core::pin::Pin;

use crate::arch::Link;
use crate::Stream;

impl Stream {

    /// Whether this is TCP.
    #[inline]
    #[must_use]
    pub const fn is_tcp ( &self ) -> bool {

        matches!(self.link, Link::Tcp(_))

    }

    /// The TCP socket itself, taken off this thread's runtime to hand to another thread; `None` for a
    /// Unix socket, or when the runtime will not let it go.
    #[inline]
    #[must_use]
    pub fn release ( self ) -> Option<StdStream> {

        match self.link {
            Link::Tcp(stream)  => stream.into_std().ok(),
            #[cfg(unix)]
            Link::Unix(_)      => None,
        }

    }

    /// Writes every byte.
    ///
    /// # Errors
    /// The peer went away or the write failed.
    #[inline]
    pub async fn send ( &mut self, bytes: &[u8] ) -> Result<()> {

        self.write_all(bytes).await.map_err(|error| Failure::from(error).context("writing to a stream"))

    }

    /// Reads what is there into `buf`; zero means the peer closed.
    ///
    /// # Errors
    /// The read failed.
    #[inline]
    pub async fn recv ( &mut self, buf: &mut [u8] ) -> Result<usize> {

        self.read(buf).await.map_err(|error| Failure::from(error).context("reading from a stream"))

    }

    /// Reads until `buf` is full.
    ///
    /// # Errors
    /// The peer closed first, or the read failed.
    #[inline]
    pub async fn fill ( &mut self, buf: &mut [u8] ) -> Result<()> {

        self.read_exact(buf).await.map(|_count| ()).map_err(|error| Failure::from(error).context("reading from a stream"))

    }

}

impl AsyncRead for Stream {

    #[inline]
    fn poll_read ( self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_> ) -> Poll<IoResult<()>> {

        match &mut self.get_mut().link {
            Link::Tcp(stream)  => Pin::new(stream).poll_read(cx, buf),
            #[cfg(unix)]
            Link::Unix(stream) => Pin::new(stream).poll_read(cx, buf),
        }

    }

}

impl AsyncWrite for Stream {

    #[inline]
    fn poll_write ( self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8] ) -> Poll<IoResult<usize>> {

        match &mut self.get_mut().link {
            Link::Tcp(stream)  => Pin::new(stream).poll_write(cx, buf),
            #[cfg(unix)]
            Link::Unix(stream) => Pin::new(stream).poll_write(cx, buf),
        }

    }

    #[inline]
    fn poll_write_vectored ( self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &[IoSlice<'_>] ) -> Poll<IoResult<usize>> {

        match &mut self.get_mut().link {
            Link::Tcp(stream)  => Pin::new(stream).poll_write_vectored(cx, bufs),
            #[cfg(unix)]
            Link::Unix(stream) => Pin::new(stream).poll_write_vectored(cx, bufs),
        }

    }

    #[inline]
    fn is_write_vectored ( &self ) -> bool {

        match &self.link {
            Link::Tcp(stream)  => stream.is_write_vectored(),
            #[cfg(unix)]
            Link::Unix(stream) => stream.is_write_vectored(),
        }

    }

    #[inline]
    fn poll_flush ( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<IoResult<()>> {

        match &mut self.get_mut().link {
            Link::Tcp(stream)  => Pin::new(stream).poll_flush(cx),
            #[cfg(unix)]
            Link::Unix(stream) => Pin::new(stream).poll_flush(cx),
        }

    }

    #[inline]
    fn poll_shutdown ( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<IoResult<()>> {

        match &mut self.get_mut().link {
            Link::Tcp(stream)  => Pin::new(stream).poll_shutdown(cx),
            #[cfg(unix)]
            Link::Unix(stream) => Pin::new(stream).poll_shutdown(cx),
        }

    }

}
