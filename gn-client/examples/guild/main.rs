mod common;
mod join;
mod keys;
mod register;
mod token;

use common::api_with_alice;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Example {
    Join,
    Keys,
    Token,
    Register {
        #[structopt(long, short)]
        operator: String
    },
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
    /// Choose which sub-example to run
    #[structopt(subcommand)]
    example: Example,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let url = format!("ws://{}:{}", opt.node_ip, opt.node_port);

    let (api, alice) = api_with_alice(url).await;

    match opt.example {
        Example::Join => join::join(api, alice).await,
        Example::Keys => keys::keys(api, alice).await,
        Example::Register { operator } => register::register(api, alice, &operator).await,
        Example::Token => token::token(api, alice).await,
    }
}
