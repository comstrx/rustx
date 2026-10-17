//! What a command-line tool stands on, the same on Linux, macOS and Windows: [`Boot`] for the
//! logger, environment and stop signal every tool starts with, [`Args`] as parsed, [`Env`] as
//! typed and layered with a dotenv file, [`Run`] for other programs with a deadline, [`System`]
//! for the machine, [`Manager`] for its packages, [`Service`] for its service manager, [`Pid`]
//! files, and [`Cargo`] for where a build put a binary. Nothing escalates privileges on its own.

#![forbid(unsafe_code)]

pub mod prelude;

mod arch;
mod faults;
mod args;
mod boot;
mod cargo;
mod env;
mod manager;
mod pid;
mod run;
mod service;
mod system;
mod tool;
mod unit;

pub use crate::arch::{Args, Boot, Cargo, Env, Faults, Flag, Manager, Named, Output, Package, Pid, Platform, Provider, Restart, Run, Service, Spec, State, System, Tool, Var};
