#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

mod key;
mod sudo;
mod transfer;

use gn_api::tx;
use sp_core::crypto::{ExposeSecret, SecretString, Zeroize};
use structopt::StructOpt;

const TX_ERROR: &str = "failed to send tx";
#[cfg(feature = "verify")]
const QUERY_ERROR: &str = "failed to execute query";

#[derive(StructOpt)]
pub enum Command {
    /// Convenience functions for key handling
    Key(KeySubCmd),
    /// Chain interactions that require sudo access
    Sudo(SudoSubCmd),
    /// Transfer funds
    Transfer {
        /// The destination account receiving the transferred amount
        #[structopt(long, short)]
        account: String,
        /// The balance to be transferred to the destination account
        #[structopt(long, short)]
        balance: u128,
    },
}

#[derive(StructOpt)]
pub enum KeySubCmd {
    /// Generates a new keypair
    Generate {
        /// The elliptic curve where the keypair is defined
        #[structopt(long, short, default_value = "sr25519")]
        curve: String,
    },
    /// Rotate session keys of the node (requires unsafe rpc calls exposed)
    Rotate,
    /// Rotate session keys and set them
    Rotset,
}

#[derive(StructOpt)]
pub enum SudoSubCmd {
    /// Oracle pallet sudo calls
    Oracle {
        #[structopt(flatten)]
        method: OracleMethod,
    },
    /// Validator pallet sudo calls
    Validator {
        #[structopt(flatten)]
        method: ValidatorMethod,
    },
}

#[derive(StructOpt)]
pub enum OracleMethod {
    /// Register an oracle operator
    Register { account: Option<String> },
    /// Deregister an oracle operator
    Deregister { account: Option<String> },
}

#[derive(StructOpt)]
pub enum ValidatorMethod {
    /// Add a validator
    Add { account: Option<String> },
    /// Remove a validator
    Remove { account: Option<String> },
}

#[derive(StructOpt)]
#[structopt(name = "Guild Network CLI")]
pub struct Opt {
    /// Set logging level
    #[structopt(short, long, default_value = "info")]
    log: String,
    /// Set node IP address
    #[structopt(short = "i", long = "node-ip", default_value = "127.0.0.1")]
    node_ip: String,
    /// Set node port number
    #[structopt(short = "p", long = "node-port", default_value = "9944")]
    node_port: String,
    /// Set operator account seed
    #[structopt(long = "seed", default_value = "//Alice")]
    seed: String,
    /// Set operator account password
    #[structopt(long = "password", default_value = "")]
    password: SecretString,
    /// CLI command to execute
    #[structopt(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() {
    let mut opt = Opt::from_args();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(opt.log)).init();

    let url = format!("ws://{}:{}", opt.node_ip, opt.node_port);
    let (api, signer) = tx::api_with_signer(url, &opt.seed, Some(opt.password.expose_secret()))
        .await
        .expect("failed to initialize client and signer");

    opt.seed.zeroize();

    log::info!("signer account: {}", signer.account_id());

    match opt.command {
        Command::Key(KeySubCmd::Generate { curve }) => {
            key::generate(&curve, opt.password.expose_secret())
        }
        Command::Key(KeySubCmd::Rotate) => {
            key::rotate(api).await;
        }
        Command::Key(KeySubCmd::Rotset) => {
            let keys = key::rotate(api.clone()).await;
            key::set(api, signer, keys).await
        }
        Command::Sudo(SudoSubCmd::Oracle { method }) => match method {
            OracleMethod::Register { account } => {
                sudo::sudo(
                    api,
                    signer,
                    account.as_deref(),
                    sudo::Method::OracleRegister,
                )
                .await
            }
            OracleMethod::Deregister { account } => {
                sudo::sudo(
                    api,
                    signer,
                    account.as_deref(),
                    sudo::Method::OracleDeregister,
                )
                .await
            }
        },
        Command::Sudo(SudoSubCmd::Validator { method }) => match method {
            ValidatorMethod::Add { account } => {
                sudo::sudo(api, signer, account.as_deref(), sudo::Method::ValidatorAdd).await
            }
            ValidatorMethod::Remove { account } => {
                sudo::sudo(
                    api,
                    signer,
                    account.as_deref(),
                    sudo::Method::ValidatorRemove,
                )
                .await
            }
        },
        Command::Transfer { account, balance } => {
            transfer::transfer(api, signer, &account, balance).await
        }
    }
}
