use crate::{runtime, AccountId, Api, SubxtError};
use gn_common::RequestIdentifier;
use parity_scale_codec::Decode;

use std::collections::BTreeMap;

pub async fn active_operators(api: Api) -> Result<Vec<AccountId>, SubxtError> {
    let operators = runtime::storage().oracle().active_operators();
    Ok(api
        .storage()
        .at(None)
        .await?
        .fetch(&operators)
        .await?
        .unwrap_or_default())
}

pub async fn is_registered(api: Api, id: &AccountId) -> Result<bool, SubxtError> {
    let operator = runtime::storage().oracle().registered_operators(id);
    Ok(api
        .storage()
        .at(None)
        .await?
        .fetch(&operator)
        .await?
        .is_some())
}

pub async fn request<T: Decode>(api: Api, id: RequestIdentifier) -> Result<T, SubxtError> {
    let key = runtime::storage().oracle().requests(id);
    let request = api
        .storage()
        .at(None)
        .await?
        .fetch(&key)
        .await?
        .ok_or_else(|| SubxtError::Other(format!("no request with id: {id}")))?;

    let request = T::decode(&mut request.data.as_slice())?;

    Ok(request)
}

pub async fn all_requests(
    api: Api,
    page_size: u32,
) -> Result<BTreeMap<RequestIdentifier, AccountId>, SubxtError> {
    let root = runtime::storage().oracle().requests_root();

    let mut map = BTreeMap::new();
    let mut iter = api.storage().at(None).await?.iter(root, page_size).await?;
    while let Some((key, value)) = iter.next().await? {
        let key_bytes = (&key.0[48..]).try_into().unwrap();
        map.insert(RequestIdentifier::from_le_bytes(key_bytes), value.operator);
    }
    Ok(map)
}
