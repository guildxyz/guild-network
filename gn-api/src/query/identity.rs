use crate::{cast, runtime, AccountId, Api, SubxtError};
use gn_common::{Authority, Identity, Prefix};

use std::collections::BTreeMap;

pub async fn identities(
    api: Api,
    user_id: &AccountId,
) -> Result<BTreeMap<Prefix, Identity>, SubxtError> {
    let identity_map = api
        .storage()
        .at(None)
        .await?
        .fetch(&runtime::storage().guild_identity().identities(user_id))
        .await?
        .ok_or(SubxtError::Other("account not registered".into()))?;

    Ok(identity_map.0.into_iter().collect())
}

pub async fn addresses(
    api: Api,
    user_id: &AccountId,
) -> Result<BTreeMap<Prefix, Vec<AccountId>>, SubxtError> {
    let address_map = api
        .storage()
        .at(None)
        .await?
        .fetch(&runtime::storage().guild_identity().addresses(user_id))
        .await?
        .ok_or(SubxtError::Other("account not registered".into()))?;

    let map = address_map
        .0
        .into_iter()
        .map(|(prefix, address_vec)| (prefix, cast::address_vec::from_runtime(address_vec)))
        .collect();

    Ok(map)
}

pub async fn authorities(api: Api, user_id: &AccountId) -> Result<[Authority; 2], SubxtError> {
    api.storage()
        .at(None)
        .await?
        .fetch(&runtime::storage().guild_identity().authorities(user_id))
        .await?
        .ok_or(SubxtError::Other("account not registered".into()))
}
