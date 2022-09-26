use client::{
    compose_extrinsic, rpc::WsRpcClient, Api, PlainTipExtrinsicParams, UncheckedExtrinsicV4,
    XtStatus,
};
//use codec::alloc::sync::mpsc::channel;
use log::{error, info, warn};
use sp_keyring::sr25519::sr25519::Pair;
use sp_keyring::AccountKeyring;

type TestApi = Api<Pair, WsRpcClient, PlainTipExtrinsicParams>;
//const PALLET: &str = "Guild";
//const METHOD: &str = "create_guild";
const PALLET: &str = "Chainlink";
const METHOD: &str = "register_operator";

pub fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let url = "ws://127.0.0.1:9944";
    let ws_client = WsRpcClient::new(url);
    let mut api = Api::new(ws_client).expect("failed to initialize client");

    // let (events_in, events_out) = channel();
    // api.subscribe_events(events_in)
    //     .expect("falied to subscribe to events");

    // TODO this will be an external function
    // where signers are randomly changed
    let signer = AccountKeyring::Alice.pair();
    api = api.set_signer(signer); // wtf - why is this not a method on `&mut self`

    if let Err(e) = try_main(api) {
        error!("{}", e);
    }
}

fn try_main(api: TestApi) -> Result<(), anyhow::Error> {
    let status = XtStatus::Ready;

    let tx: UncheckedExtrinsicV4<_, _> = compose_extrinsic!(api, PALLET, METHOD);

    if let Some(tx_hash) = api.send_extrinsic(tx.hex_encode(), status)? {
        info!("blockhash: {}", tx_hash)
    } else {
        warn!("transaction not yet finalized")
    }

    Ok(())
}
