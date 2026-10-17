use net::{Address, Bind, Bound, Listener, Peer, Stream};
use tokio::sync::mpsc::{Sender, channel};
use core::iter::{once, repeat_with};
use error::{Failure, Result};
use core::net::SocketAddr;

use crate::arch::{Door, Handoff, Source};

use crate::Faults;

const DEPTH: usize = 1024;

impl Door {

    fn prepare ( address: &Address, bind: Bind ) -> Result<Self> {

        let bound = Bound::prepare(address, bind)?;
        let local = bound.local().clone();

        Ok(Self { bound: Some(bound), live: None, local })

    }

    fn listener ( &mut self ) -> Result<&Listener> {

        if let Some(bound) = self.bound.take() { self.live = Some(bound.listen()?); }

        self.live.as_ref().ok_or_else(|| Failure::internal("the door has no listener"))

    }

}

impl Source {

    pub(crate) fn spread ( address: &Address, bind: Bind, count: usize ) -> Result<Vec<Self>> {

        if count <= 1 { return Ok(vec![Self::Socket(Door::prepare(address, bind)?)]); }

        if Self::shareable(address) { return Self::shared(address, bind, count); }

        let door = Door::prepare(address, bind)?;
        let mut sources = Vec::with_capacity(count);
        let mut senders = Vec::with_capacity(count - 1);

        for _ in 1..count {

            let ( sender, receiver ) = channel::<Handoff>(DEPTH);

            senders.push(sender);
            sources.push(Self::Channel(receiver));

        }

        sources.insert(0, Self::Fanout { door, senders, next: 0 });

        Ok(sources)

    }

    fn shared ( address: &Address, bind: Bind, count: usize ) -> Result<Vec<Self>> {

        let first = Door::prepare(address, bind.reuse(true))?;
        let local = first.local.clone();
        let rest  = repeat_with(|| Door::prepare(&local, bind.reuse(true)).map(Self::Socket)).take(count - 1);

        once(Ok(Self::Socket(first))).chain(rest).collect()

    }

    pub(crate) const fn local ( &self ) -> Option<&Address> {

        match self {
            Self::Socket(door) | Self::Fanout { door, .. } => Some(&door.local),
            Self::Channel(_)                               => None,
        }

    }

    pub(crate) async fn accept ( &mut self ) -> Result<( Stream, Peer )> {

        match self {
            Self::Socket(door)                      => door.listener()?.accept().await,
            Self::Channel(receiver)                 => Self::adopt(receiver.recv().await.ok_or_else(|| Faults::ACCEPTOR_GONE.fail())?),
            Self::Fanout { door, senders, next }    => Self::spread_one(door.listener()?, senders, next).await,
        }

    }

    async fn spread_one ( listener: &Listener, senders: &[Sender<Handoff>], next: &mut usize ) -> Result<( Stream, Peer )> {

        let mut kept = None;

        while kept.is_none() { kept = Self::handoff(listener.accept().await?, senders, next); }

        kept.ok_or_else(|| Failure::internal("the fan-out kept nothing"))

    }

    fn handoff ( accepted: ( Stream, Peer ), senders: &[Sender<Handoff>], next: &mut usize ) -> Option<( Stream, Peer )> {

        let turn = *next;

        *next = (turn + 1) % (senders.len() + 1);

        let Some(target) = turn.checked_sub(1).and_then(|index| senders.get(index)) else { return Some(accepted); };
        let ( stream, peer ) = accepted;

        if !stream.is_tcp() { return Some(( stream, peer )); }

        let Some(addr) = Self::socket(&peer) else { return Some(( stream, peer )); };
        let std = stream.release()?;

        match target.try_send(( std, addr )) {
            Ok(())        => None,
            Err(rejected) => Self::adopt(rejected.into_inner()).ok(),
        }

    }

    fn adopt ( ( std, addr ): Handoff ) -> Result<( Stream, Peer )> {

        Listener::adopt(std, addr)

    }

    #[expect(clippy::wildcard_enum_match_arm, reason = "`Address` is non-exhaustive and only TCP can be handed off")]
    const fn socket ( peer: &Peer ) -> Option<SocketAddr> {

        match peer.address() {
            Address::Tcp(addr) => Some(*addr),
            _                  => None,
        }

    }

    const fn shareable ( address: &Address ) -> bool {

        cfg!(any(target_os = "linux", target_os = "android")) && matches!(address, Address::Tcp(_))

    }

}
