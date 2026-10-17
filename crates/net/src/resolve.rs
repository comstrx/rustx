use error::Result;
use tokio::net::lookup_host;
use core::net::SocketAddr;

use crate::{Faults, Resolve};

impl Resolve {

    /// Every address `host` resolves to, with `port`, `IPv4` before `IPv6`.
    ///
    /// # Errors
    /// `Unavailable` keyed `no_address` when nothing resolves.
    #[inline]
    pub async fn lookup ( host: &str, port: u16 ) -> Result<Vec<SocketAddr>> {

        let found = lookup_host(( host, port )).await.map_err(|error| Faults::NO_ADDRESS.because("the name could not be resolved").caused_by(error).context(format!("resolving {host}")))?;
        let mut addrs: Vec<SocketAddr> = found.collect();

        addrs.sort_by_key(SocketAddr::is_ipv6);

        if addrs.is_empty() { return Err(Faults::NO_ADDRESS.fail().context(host.to_owned())); }

        Ok(addrs)

    }

}
