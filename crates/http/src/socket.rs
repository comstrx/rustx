use hyper::header::{CONNECTION, CONTENT_LENGTH, ORIGIN, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION, UPGRADE};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use bytes::{Buf, Bytes as Shared, BytesMut};
use memory::{Bytes, Space as _, Stack};
use error::{Failure, Result};
use std::io::Error as IoError;
use hyper_util::rt::TokioIo;
use hyper::upgrade::on;
use crypto::Digest;
use parse::List;

use crate::{Faults, Message, Request, Response, Socket, Upgrade};
use crate::arch::{Extent, Frame, Step};

const GUID: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

impl Socket {

    /// Answers a same-origin `WebSocket` handshake: the `101` to return, and the upgrade to wait for after it went out.
    ///
    /// # Errors
    /// `Invalid` keyed `not_websocket` when the request is not a version-13 `WebSocket` handshake; `Forbidden`
    /// keyed `cross_origin` when a browser sent it from another origin.
    #[inline]
    pub fn accept ( request: &mut Request ) -> Result<( Response, Upgrade )> {

        let upgrade    = request.field(&UPGRADE).is_some_and(|value| value.eq_ignore_ascii_case("websocket"));
        let connection = request.field(&CONNECTION).is_some_and(|value| List::has(value, "upgrade"));
        let version    = request.field(&SEC_WEBSOCKET_VERSION).is_some_and(|value| value.trim() == "13");
        let key        = request.field(&SEC_WEBSOCKET_KEY).map(str::trim).filter(|key| !key.is_empty()).map(str::to_owned);

        let Some(key) = key.filter(|_| upgrade && connection && version) else { return Err(Faults::NOT_WEBSOCKET.fail()); };

        Self::origin(request)?;

        let accept  = Digest::base64(&Digest::sha1(&[key.as_bytes(), GUID].concat()));
        let pending = on(request.inner_mut());

        let mut response = Response::status(101).with(UPGRADE, "websocket").with(CONNECTION, "upgrade").with(SEC_WEBSOCKET_ACCEPT, &accept);

        response.headers_mut().remove(CONTENT_LENGTH);

        Ok(( response, Upgrade { pending, limit: 1 << 20 } ))

    }

    fn origin ( request: &Request ) -> Result<()> {

        let Some(origin) = request.field(&ORIGIN) else { return Ok(()); };

        let same = origin
            .split_once("://")
            .zip(request.host())
            .is_some_and(|( ( _, authority ), host )| authority.eq_ignore_ascii_case(host));

        Faults::CROSS_ORIGIN.ensure(same)

    }

    /// The next message; `Close` once when the peer closes, then `None`.
    ///
    /// # Errors
    /// The peer broke the protocol, sent more than the limit, or went away.
    #[inline]
    pub async fn recv ( &mut self ) -> Result<Option<Message>> {

        if self.closed { return Ok(None); }

        let mut pending: Option<( u8, BytesMut )> = None;
        let mut outcome = None;

        while outcome.is_none() {

            let frame = self.frame().await?;

            outcome = self.consume(frame, &mut pending).await?;

        }

        Ok(outcome.flatten())

    }

    /// Sends one message.
    ///
    /// # Errors
    /// The socket is closed, or the write failed.
    #[inline]
    pub async fn send ( &mut self, message: Message ) -> Result<()> {

        if self.closed { return Err(Faults::SOCKET_CLOSED.fail()); }

        let ( opcode, data ) = match message {
            Message::Text(text)    => ( 0x1, Shared::from(text) ),
            Message::Binary(bytes) => ( 0x2, Shared::from(bytes) ),
            Message::Ping(bytes)   => ( 0x9, Shared::from(bytes) ),
            Message::Pong(bytes)   => ( 0xA, Shared::from(bytes) ),
            Message::Close(code)   => ( 0x8, code.map_or_else(Shared::new, |code| Shared::copy_from_slice(&code.to_be_bytes())) ),
        };

        self.write(opcode, &data).await

    }

    /// Sends a close frame with this code; the next `recv` returns `None`.
    ///
    /// # Errors
    /// The write failed.
    #[inline]
    pub async fn close ( &mut self, code: u16 ) -> Result<()> {

        if self.closed { return Ok(()); }

        self.write(0x8, &code.to_be_bytes()).await?;
        self.closed = true;

        Ok(())

    }

    /// Whether the close handshake happened.
    #[inline]
    #[must_use]
    pub const fn is_closed ( &self ) -> bool {

        self.closed

    }

    async fn write ( &mut self, opcode: u8, data: &[u8] ) -> Result<()> {

        let mut head = Stack::<10>::new();

        head.push_byte(0x80 | opcode)?;
        Self::length(&mut head, data.len())?;

        let mut chained = Buf::chain(head.as_bytes(), data);

        self.link.write_all_buf(&mut chained).await.map_err(Self::carried)?;
        self.link.flush().await.map_err(Self::carried)

    }

    fn length ( head: &mut Stack<10>, len: usize ) -> Result<()> {

        let len64 = u64::try_from(len).unwrap_or(u64::MAX);

        match len {
            0..=125         => head.push_byte(u8::try_from(len).unwrap_or(125)).map(|_| ()),
            126..=65_535    => head.push_byte(126)?.push(&u16::try_from(len).unwrap_or(u16::MAX).to_be_bytes()).map(|_| ()),
            _               => head.push_byte(127)?.push(&len64.to_be_bytes()).map(|_| ()),
        }

    }

    async fn frame ( &mut self ) -> Result<Frame> {

        let mut parsed = self.parse()?;

        while parsed.is_none() {

            self.fill().await?;
            parsed = self.parse()?;

        }

        parsed.ok_or_else(|| Failure::internal("the frame vanished"))

    }

    async fn fill ( &mut self ) -> Result<()> {

        self.buffer.reserve(4096);

        let came = self.link.read_buf(&mut self.buffer).await.map_err(Self::carried)?;

        Faults::SOCKET_GONE.ensure(came > 0)

    }

    fn parse ( &mut self ) -> Result<Option<Frame>> {

        let Some(&[first, second]) = self.buffer.first_chunk::<2>() else { return Ok(None); };

        Self::holds(first & 0x70 == 0, "a reserved bit is set")?;
        Self::holds(second & 0x80 != 0, "a client frame is not masked")?;

        let ( extra, payload ) = Self::extent(&self.buffer, second & 0x7F)?;
        let Some(payload) = payload else { return Ok(None); };
        let header = 2 + extra + 4;

        Faults::SOCKET_TOO_LARGE.ensure(payload <= self.limit)?;

        if self.buffer.len() < header + payload { return Ok(None); }

        let head = self.buffer.split_to(header);
        let key  = head.get(header - 4..).and_then(|key| key.first_chunk::<4>()).copied().unwrap_or_default();
        let mut data = self.buffer.split_to(payload);

        for ( index, byte ) in data.iter_mut().enumerate() { *byte ^= key.get(index % 4).copied().unwrap_or_default(); }

        Ok(Some(Frame { fin: first & 0x80 != 0, opcode: first & 0x0F, data: data.freeze() }))

    }

    fn extent ( buffer: &[u8], short: u8 ) -> Result<Extent> {

        match short {
            126 => Ok(( 2, buffer.get(2..4).and_then(|bytes| bytes.first_chunk::<2>()).map(|bytes| usize::from(u16::from_be_bytes(*bytes))) )),
            127 => buffer.get(2..10).and_then(|bytes| bytes.first_chunk::<8>()).map_or(Ok(( 8, None )), |bytes| usize::try_from(u64::from_be_bytes(*bytes)).map(|len| ( 8, Some(len) )).map_err(|error| Faults::SOCKET_TOO_LARGE.because("a frame passes what this machine holds").caused_by(error))),
            len => Ok(( 0, Some(usize::from(len)) )),
        }

    }

    async fn consume ( &mut self, frame: Frame, pending: &mut Option<( u8, BytesMut )> ) -> Result<Step> {

        match frame.opcode {
            0x0        => self.continued(&frame, pending),
            0x1 | 0x2  => Self::started(frame, pending),
            0x8        => self.closing(&frame.data).await,
            0x9        => self.write(0xA, &frame.data).await.map(|()| Some(Some(Message::Ping(Bytes::from(frame.data))))),
            0xA        => Ok(Some(Some(Message::Pong(Bytes::from(frame.data))))),
            _          => Err(Faults::WEBSOCKET_PROTOCOL.because("an unknown opcode")),
        }

    }

    fn started ( frame: Frame, pending: &mut Option<( u8, BytesMut )> ) -> Result<Step> {

        Self::holds(pending.is_none(), "a new message began inside another")?;

        if frame.fin { return Self::deliver(frame.opcode, frame.data); }

        *pending = Some(( frame.opcode, BytesMut::from(frame.data) ));

        Ok(None)

    }

    fn continued ( &self, frame: &Frame, pending: &mut Option<( u8, BytesMut )> ) -> Result<Step> {

        let Some(( opcode, mut data )) = pending.take() else { return Err(Faults::WEBSOCKET_PROTOCOL.because("a continuation without a start")); };

        Faults::SOCKET_TOO_LARGE.ensure(data.len() + frame.data.len() <= self.limit)?;
        data.extend_from_slice(&frame.data);

        if frame.fin { return Self::deliver(opcode, data.freeze()); }

        *pending = Some(( opcode, data ));

        Ok(None)

    }

    async fn closing ( &mut self, data: &Shared ) -> Result<Step> {

        let code = data.first_chunk::<2>().map(|bytes| u16::from_be_bytes(*bytes));

        if !self.closed {

            self.write(0x8, data.get(..2).unwrap_or_default()).await?;
            self.closed = true;

            return Ok(Some(Some(Message::Close(code))));

        }

        Ok(Some(None))

    }

    fn deliver ( opcode: u8, data: Shared ) -> Result<Step> {

        if opcode == 0x2 { return Ok(Some(Some(Message::Binary(Bytes::from(data))))); }

        String::from_utf8(data.to_vec()).map(|text| Some(Some(Message::Text(text)))).map_err(|error| Faults::WEBSOCKET_PROTOCOL.because("a text message is not UTF-8").caused_by(error.utf8_error()))

    }

    fn holds ( condition: bool, detail: &'static str ) -> Result<()> {

        if condition { Ok(()) } else { Err(Faults::WEBSOCKET_PROTOCOL.because(detail)) }

    }

    fn carried ( error: IoError ) -> Failure {

        Faults::SOCKET_GONE.fail().caused_by(error).context("on the websocket")

    }

}

impl Upgrade {

    /// How many bytes one message may hold.
    #[inline]
    #[must_use]
    pub const fn limit ( mut self, bytes: usize ) -> Self {

        self.limit = bytes;

        self

    }

    /// The socket, once the `101` has gone out and hyper handed the connection over.
    ///
    /// # Errors
    /// The connection ended before the upgrade.
    #[inline]
    pub async fn ready ( self ) -> Result<Socket> {

        let upgraded = self.pending.await.map_err(|error| Faults::SOCKET_GONE.because("the connection did not upgrade").caused_by(error))?;

        Ok(Socket { link: TokioIo::new(upgraded), buffer: BytesMut::with_capacity(4096), limit: self.limit, closed: false })

    }

}

impl Message {

    /// A text message.
    #[inline]
    #[must_use]
    pub fn text ( text: impl Into<String> ) -> Self {

        Self::Text(text.into())

    }

    /// A binary message.
    #[inline]
    #[must_use]
    pub fn binary ( bytes: impl Into<Bytes> ) -> Self {

        Self::Binary(bytes.into())

    }

}

