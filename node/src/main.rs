//! Substrate Node Template CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[cfg(feature = "node")]
#[macro_use]
mod service;
mod cli;
mod command;
#[cfg(feature = "runtime-benchmarks")]
mod command_benchmark;
#[cfg(feature = "node")]
mod rpc;

fn main() -> sc_cli::Result<()> {
    command::run()
}