mod common;
mod join;
mod sign;

use common::api_with_alice;

use structopt::StructOpt;

use std::str::FromStr;

#[derive(Debug, StructOpt)]
enum Example {
    Join,
    Sign,
}

impl FromStr for Example {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "join" => Ok(Self::Join),
            "sign" => Ok(Self::Sign),
            _ => Err(format!("no example with name {s}")),
        }
    }
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
    #[structopt(short, long, default_value = "join")]
    example: Example,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let url = format!("ws://{}:{}", opt.node_ip, opt.node_port);

    let (api, alice) = api_with_alice(url).await;

    match opt.example {
        Example::Join => join::join(api, alice).await,
        Example::Sign => sign::sign(api, alice).await,
    }
}
