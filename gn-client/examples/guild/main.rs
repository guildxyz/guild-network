mod common;
mod fund;
mod join;
mod key;
mod sudo;
mod token;

use gn_client::tx;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Example {
    Fund {
        #[structopt(long, short)]
        account: String,
        #[structopt(long, short)]
        balance: u128,
    },
    Join,
    Key(Key),
    Token,
    Sudo {
        #[structopt(long, short)]
        pallet: String,
        #[structopt(long, short)]
        method: String,
        #[structopt(long, short)]
        account: Option<String>,
    },
}

#[derive(Debug, StructOpt)]
enum Key {
    Generate {
        #[structopt(long, short, default_value = "sr25519")]
        curve: String,
        #[structopt(long, short)]
        password: Option<String>,
    },
    Rotate,
    Rotset,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Client params",
    about = "Advanced parameters for the Substrate client."
)]
struct Opt {
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
    /// Choose which sub-example to run
    #[structopt(subcommand)]
    example: Example,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let url = format!("ws://{}:{}", opt.node_ip, opt.node_port);

    let (api, signer) = tx::api_with_signer(url, &opt.seed, opt.password.as_deref())
        .await
        .expect("failed to initialize client and signer");

    println!("signer pubkey: {}", signer.account_id());

    match opt.example {
        Example::Fund { account, balance } => fund::fund(api, signer, &account, balance).await,
        Example::Join => join::join(api, signer).await,
        Example::Key(Key::Generate { curve, password }) => {
            key::generate(&curve, password.as_deref())
        }
        Example::Key(Key::Rotate) => {
            key::rotate(api).await;
        }
        Example::Key(Key::Rotset) => {
            let keys = key::rotate(api.clone()).await;
            key::set(api, signer, keys).await
        }
        Example::Sudo {
            pallet,
            method,
            account,
        } => {
            sudo::sudo(
                api,
                signer,
                &pallet.to_lowercase(),
                &method.to_lowercase(),
                account.as_deref(),
            )
            .await
        }
        Example::Token => token::token(api, signer).await,
    }
}
