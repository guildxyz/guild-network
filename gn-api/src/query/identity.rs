use crate::{cast, runtime, AccountId, Api, SubxtError};
use gn_common::{Identity, Prefix};

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

    Ok(cast::identity_map::from_runtime(identity_map))
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

    Ok(cast::address_map::from_runtime(address_map))
}
