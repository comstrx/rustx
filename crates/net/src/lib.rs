//! The transport every protocol crate stands on: an [`Address`] that is TCP, Unix or a name, a
//! [`Listener`] on a socket tuned for servers whose [`Hook`]s admit or refuse a peer at accept
//! time, a [`Stream`] over either transport, [`Dial`] with a deadline and [`Resolve`] for names.

#![forbid(unsafe_code)]

extern crate alloc;

pub mod prelude;

mod address;
mod arch;
mod faults;
mod dial;
mod hook;
mod listener;
mod resolve;
mod socket;
mod stream;

pub use crate::arch::{Address, Bind, Bound, Dial, Faults, Hook, Listener, Peer, Protocol, Resolve, Stream};
