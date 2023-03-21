use gn_client::{
    tx::{self, Signer},
    AccountId, Api,
};

use std::str::FromStr;
use std::sync::Arc;

pub enum Method {
    OracleRegister,
    OracleDeregister,
    ValidatorAdd,
    ValidatorRemove,
}

pub async fn sudo(api: Api, signer: Arc<Signer>, maybe_operator: Option<&str>, method: Method) {
    let account_id = if let Some(operator) = maybe_operator {
        AccountId::from_str(operator).expect("invalid operator id string")
    } else {
        signer.account_id().clone()
    };

    let payload = match method {
        Method::OracleRegister => tx::register_operator(&account_id),
        Method::OracleDeregister => tx::deregister_operator(&account_id),
        Method::ValidatorAdd => tx::add_validator(&account_id),
        Method::ValidatorRemove => tx::remove_validator(&account_id),
    };

    tx::send_tx_in_block(api, &tx::sudo(payload), signer)
        .await
        .expect("failed to send tx");
}
