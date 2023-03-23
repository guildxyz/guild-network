#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

mod evm;
mod guild;
mod key;
mod oracle;
mod sudo;
mod transfer;

use gn_client::tx;
use structopt::StructOpt;

use std::path::PathBuf;

const TX_ERROR: &str = "failed to send tx";
const QUERY_ERROR: &str = "failed to execute query";

#[derive(StructOpt)]
pub enum Command {
    /// Convenience functions for key handling
    Key(KeySubCmd),
    /// Start and oracle node
    Oracle {
        /// Activate operator before starting to listen to events
        #[structopt(long)]
        activate: bool,
    },
    /// Chain interactions that require sudo access
    Sudo(SudoSubCmd),
    /// Guild-related on-chain interactions
    Guild(GuildSubCmd),
    /// Transfer funds
    Transfer {
        /// The destination account receiving the transferred amount
        #[structopt(long, short)]
        account: String,
        /// The balance to be transferred to the destination account
        #[structopt(long, short)]
        balance: u128,
    },
    /// Evm related convenience functions
    Evm(EvmSubCmd),
}

#[derive(StructOpt)]
pub enum EvmSubCmd {
    AddressGen {
        /// Output file path
        #[structopt(long, short, default_value = "/tmp/addresses.txt")]
        output: PathBuf,
        /// Number of accounts to generate
        #[structopt(long, short, default_value = "100")]
        num: usize,
        /// Initial seed
        #[structopt(long, short, default_value = "guild network")]
        seed: String,
    },
}

#[derive(StructOpt)]
pub enum KeySubCmd {
    /// Generates a new keypair
    Generate {
        /// The elliptic curve where the keypair is defined
        #[structopt(long, short, default_value = "sr25519")]
        curve: String,
        /// Optional password for the generated keypair
        #[structopt(long, short)]
        password: Option<String>,
    },
    /// Rotate session keys of the node (requires unsafe rpc calls exposed)
    Rotate,
    /// Rotate session keys and set them
    Rotset,
}

// TODO add create guild and create role where input can be parsed from a
// json file
#[derive(StructOpt)]
pub enum GuildSubCmd {
    /// Register an identity on Guild Network
    Register(Identity),
    /// Join a specific role in a guild
    Join {
        /// Guild name
        #[structopt(long, short)]
        guild: String,
        /// Role name
        #[structopt(long, short)]
        role: String,
        /// Index among the user's registered identities
        #[structopt(long, short, requires("leaf"))]
        id: Option<u8>,
        /// Index of identity in the role's allowlist
        #[structopt(long, short, requires("id"))]
        leaf: Option<usize>,
    },
}

#[derive(StructOpt)]
pub enum Identity {
    /// Discord identity handle
    Discord {
        id: String,
        #[structopt(default_value = "0")]
        index: u8,
    },
    /// Telegram identity handle
    Telegram {
        id: String,
        #[structopt(default_value = "0")]
        index: u8,
    },
    /// EVM specific address and respective signature
    Evm {
        address: String,
        signature: String,
        #[structopt(default_value = "0")]
        index: u8,
    },
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
    #[structopt(long = "password")]
    password: Option<String>,
    /// CLI command to execute
    #[structopt(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(opt.log)).init();

    let url = format!("ws://{}:{}", opt.node_ip, opt.node_port);
    let (api, signer) = tx::api_with_signer(url, &opt.seed, opt.password.as_deref())
        .await
        .expect("failed to initialize client and signer");

    log::info!("signer account: {}", signer.account_id());

    match opt.command {
        Command::Evm(EvmSubCmd::AddressGen { output, num, seed }) => {
            evm::generate_test_accounts(output, num, &seed)
        }
        Command::Guild(GuildSubCmd::Register(identity)) => {
            guild::register_identity(api, signer, identity).await
        }
        Command::Guild(GuildSubCmd::Join {
            guild,
            role,
            id,
            leaf,
        }) => {
            let indices = id
                .zip(leaf)
                .map(|(i, l)| guild::ProofIndices { id: i, leaf: l });
            guild::join(api, signer, guild, role, indices).await
        }
        Command::Key(KeySubCmd::Generate { curve, password }) => {
            key::generate(&curve, password.as_deref())
        }
        Command::Key(KeySubCmd::Rotate) => {
            key::rotate(api).await;
        }
        Command::Key(KeySubCmd::Rotset) => {
            let keys = key::rotate(api.clone()).await;
            key::set(api, signer, keys).await
        }
        Command::Oracle { activate } => oracle::oracle(api, signer, activate).await,
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
