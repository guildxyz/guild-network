#[cfg(not(feature = "external-oracle"))]
mod mock;
#[cfg(not(feature = "external-oracle"))]
pub use mock::send_dummy_oracle_answers;

use gn_api::query;
use gn_api::tx::Signer;
use gn_api::{AccountId, Api};
use gn_test_data::*;

use std::sync::Arc;

pub async fn init_operators(api: Api, _root: Arc<Signer>) -> Vec<Arc<Signer>> {
    #[cfg(not(feature = "external-oracle"))]
    let operators = mock::init_dummy_operators(api.clone(), _root).await;

    #[cfg(feature = "external-oracle")]
    let operators = Vec::new();

    let _active_operators = wait_for_active_operators(api.clone()).await;

    #[cfg(not(feature = "external-oracle"))]
    for active in &_active_operators {
        assert!(operators.iter().any(|op| op.account_id() == active));
    }

    operators
}

async fn wait_for_active_operators(api: Api) -> Vec<AccountId> {
    let mut i = 0;
    loop {
        let active_operators = query::active_operators(api.clone())
            .await
            .expect("failed to fetch active operators");
        if active_operators.is_empty() {
            i += 1;
            println!("waiting for active operators");
            if i == RETRIES {
                panic!("no active operators found");
            }
            tokio::time::sleep(std::time::Duration::from_millis(SLEEP_DURATION_MS)).await;
        } else {
            println!("found an active operator");
            return active_operators;
        }
    }
}

pub async fn wait_for_oracle_answers(api: Api) {
    let mut i = 0;
    loop {
        let oracle_requests = query::oracle_requests(api.clone(), PAGE_SIZE)
            .await
            .expect("failed to fetch oracle requests");
        if !oracle_requests.is_empty() {
            i += 1;
            if i == RETRIES {
                panic!("ran out of retries while checking oracle requests")
            }
            tokio::time::sleep(std::time::Duration::from_millis(SLEEP_DURATION_MS)).await;
        } else {
            break;
        }
    }
}
