mod common;
mod join;
mod token;

use gn_api::tx;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Example {
    Join,
    Token,
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
        Example::Join => join::join(api, signer).await,
        Example::Token => token::token(api, signer).await,
    }
}
