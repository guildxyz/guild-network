use gn_client::transactions;
use gn_client::{AccountId, Api, PreparedMsgWithParams, RuntimeIdentityWithAuth};

pub async fn register(
    api: Api,
    account_id: &AccountId,
    evm_address: Option<String>,
    evm_signature: Option<String>,
    discord: Option<String>,
    telegram: Option<String>,
) -> Result<PreparedMsgWithParams, anyhow::Error> {
    let mut identities = Vec::new();

    match (evm_address, evm_signature) {
        (Some(address), Some(signature)) => {
            let mut evm_address_bytes = [0u8; 20];
            let mut evm_signature_bytes = [0u8; 65];
            hex::decode_to_slice(&address, &mut evm_address_bytes)?;
            hex::decode_to_slice(&signature, &mut evm_signature_bytes)?;
            identities.push(RuntimeIdentityWithAuth::EvmChain(
                evm_address_bytes,
                evm_signature_bytes,
            ));
        }
        (None, None) => {}
        _ => return Err(anyhow::anyhow!("missing address or signature")),
    }

    if let Some(dc_id) = discord {
        let dc_id_u64 = dc_id.parse::<u64>()?;
        identities.push(RuntimeIdentityWithAuth::Discord(dc_id_u64, ()));
    }

    if let Some(tg_id) = telegram {
        let tg_id_u64 = tg_id.parse::<u64>()?;
        identities.push(RuntimeIdentityWithAuth::Telegram(tg_id_u64, ()));
    }

    let tx_payload = transactions::register(identities);

    let prepared = api
        .tx()
        .prepare_unsigned(&tx_payload, account_id, Default::default())
        .await?;

    Ok(prepared)
}
