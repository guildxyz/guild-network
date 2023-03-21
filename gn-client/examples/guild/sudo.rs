use gn_client::{
    tx::{self, Signer},
    AccountId, Api,
};

use std::str::FromStr;
use std::sync::Arc;

pub async fn sudo(
    api: Api,
    root: Arc<Signer>,
    pallet: &str,
    method: &str,
    maybe_operator: Option<&str>,
) {
    let account_id = if let Some(operator) = maybe_operator {
        AccountId::from_str(operator).expect("invalid operator id string")
    } else {
        root.account_id().clone()
    };

    let payload = match (pallet, method) {
        ("oracle", "register") => tx::register_operator(&account_id),
        ("oracle", "deregister") => tx::deregister_operator(&account_id),
        ("validator", "add") => tx::add_validator(&account_id),
        ("validator", "remove") => tx::remove_validator(&account_id),
        _ => unimplemented!(),
    };

    tx::send_tx_in_block(api, &tx::sudo(payload), root)
        .await
        .unwrap();
    println!("{pallet} {method} {account_id} submitted successfully");
}
