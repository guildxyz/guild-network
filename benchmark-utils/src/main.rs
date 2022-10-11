use client::{
    compose_extrinsic, rpc::WsRpcClient, AccountId, Api, MultiAddress, Pair,
    PlainTipExtrinsicParams, UncheckedExtrinsicV4, XtStatus,
};
//use codec::alloc::sync::mpsc::channel;
use codec::Encode;
use log::{error, info, warn};
use sp_keyring::sr25519::sr25519::Pair as SrPair;
use sp_keyring::AccountKeyring;

type TestApi = Api<SrPair, WsRpcClient, PlainTipExtrinsicParams>;
//const PALLET: &str = "Guild";
//const METHOD: &str = "create_guild";
const CHAINLINK_PALLET: &str = "Chainlink";
const CHAINLINK_METHOD: &str = "register_operator";

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
    api = api.set_signer(signer);
    // generate new keypairs
    let mut seed = [10u8; 32];
    let operators = (0..10)
        .map(|_| {
            let (keypair, _secret_seed) = SrPair::from_entropy(&seed, None);
            seed[0] += 1;
            keypair
        })
        .collect::<Vec<SrPair>>();

    let amount = 100_000u128;
    for operator in &operators {
        fund_account(&api, operator, amount).expect("failed to fund account");
    }

    for operator in &operators {
        api = api.set_signer(operator.clone());
        if let Err(e) = register_operator(&api) {
            error!("{}", e);
        }
    }
}

fn register_operator(api: &TestApi) -> Result<(), anyhow::Error> {
    let tx: UncheckedExtrinsicV4<_, _> =
        compose_extrinsic!(api, CHAINLINK_PALLET, CHAINLINK_METHOD);
    send_tx(api, tx, XtStatus::Ready)
}

fn fund_account(api: &TestApi, recipient: &SrPair, amount: u128) -> Result<(), anyhow::Error> {
    let recipient_account_id = AccountId::new(*recipient.public().as_ref());
    let tx = api.balance_transfer(MultiAddress::Id(recipient_account_id), amount);
    send_tx(api, tx, XtStatus::InBlock)
}

fn send_tx<Call: Encode, SignedExtra: Encode>(
    api: &TestApi,
    tx: UncheckedExtrinsicV4<Call, SignedExtra>,
    status: XtStatus,
) -> Result<(), anyhow::Error> {
    if let Some(tx_hash) = api.send_extrinsic(tx.hex_encode(), status)? {
        info!("blockhash: {}", tx_hash)
    } else {
        warn!("transaction not yet finalized")
    }

    Ok(())
}
