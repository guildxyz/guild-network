pub mod identities;
pub mod requirements;

#[macro_export]
macro_rules! address {
    ($addr:expr) => {{
        <ethereum_types::Address as std::str::FromStr>::from_str($addr)
            .expect(&format!("invalid address {}", $addr))
    }};
}

// TODO this will go to guild_network_client
/*
pub async fn process_request<'a>(
    request_id: u64,
    minimum_balance: U256,
    data: &[u8],
) -> Result<DynamicTxPayload<'a>, anyhow::Error> {
    let _data = hex::encode(&data[1..]);

    let balance = U256::from_dec_str("1000").expect("This should be fine");
    let access = balance >= minimum_balance;

    Ok(subxt::dynamic::tx(
        "Chainlink",
        "callback",
        vec![
            Value::u128(request_id as u128),
            Value::from_bytes([access as u8]),
        ],
    ))
}

#[cfg(all(test, not(feature = "flood-tests")))]
mod client_tests {
    use super::{process_request, U256};
    use crate::{data::OracleRequestArgs, types::Api};

    use ethock_lib::server::{Entry, ServerType};
    use sp_keyring::AccountKeyring::Alice;
    use std::{thread, time::Duration};
    use subxt::tx::PairSigner;

    const IP: &str = "127.0.0.1";
    const PORT: u16 = 9001;

    #[tokio::test]
    async fn test_process_request() {
        thread::spawn(|| {
            Entry::new(ServerType::WS, &format!("{IP}:{PORT}"), "info").serve_silent()
        });
        thread::sleep(Duration::from_millis(100));

        let api = Api::from_url(&format!("ws://{IP}:{PORT}")).await.unwrap();
        let signer = PairSigner::new(Alice.pair());
        let request = OracleRequestArgs::dummy();
        let minimum_balance = U256::from_dec_str("12345").expect("This should be fine");

        let tx = process_request(request.request_id, minimum_balance, &request.data)
            .await
            .unwrap();

        assert!(api.tx().sign_and_submit_default(&tx, &signer).await.is_ok());
    }
}
*/
