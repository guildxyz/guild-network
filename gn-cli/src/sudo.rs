#[cfg(feature = "verify")]
use gn_client::query;
use gn_client::{
    tx::{self, Signer},
    AccountId, Api,
};

use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone, Copy)]
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

    #[cfg(not(feature = "verify"))]
    tx::send::ready(api, &tx::sudo(payload), signer)
        .await
        .expect("failed to send tx");

    #[cfg(feature = "verify")]
    {
        tx::send::in_block(api.clone(), &tx::sudo(payload), signer)
            .await
            .expect("failed to send tx");

        match method {
            Method::OracleRegister => {
                assert!(query::is_operator_registered(api, &account_id)
                    .await
                    .expect("query failed"));
            }
            Method::OracleDeregister => {
                assert!(!query::is_operator_registered(api, &account_id)
                    .await
                    .expect("query failed"));
            }
            Method::ValidatorAdd => {
                assert!(query::is_validator_added(api, &account_id)
                    .await
                    .expect("query failed"));
            }
            Method::ValidatorRemove => {
                assert!(!query::is_validator_added(api, &account_id)
                    .await
                    .expect("query failed"));
            }
        }
    }
}
