use client::{compose_extrinsic, Api, UncheckedExtrinsicV4, XtStatus};
use log::{info, warn};

const PALLET: &str = "Chainlink";
const CALL: &str = "callback";

pub fn send_transaction() {
    let request_id = 0_u64;
    let data = todo!();
    let status = todo!();
    let api: Api<_, _, _> = todo!();

    let tx: UncheckedExtrinsicV4<_, _> =
        compose_extrinsic!(api, PALLET, CALL, request_id, data);

    if let Some(tx_hash) = api.send_extrinsic(tx.hex_encode(), status) {
        info!("blockhash: {}", tx_hash)
    } else {
        warn!("transaction not yet finalized")
    }
    todo!()
}
