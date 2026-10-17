use crate::{Hook, Peer};

impl <F: Fn(&Peer) -> bool + Send + Sync> Hook for F {

    #[inline]
    fn admit ( &self, peer: &Peer ) -> bool {

        self(peer)

    }

}
