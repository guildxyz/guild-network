use crate::{runtime, AccountId, Api, Hash, JoinRequest};
use guild_network_common::{GuildName, RequestIdentifier, RoleName};
use subxt::storage::address::{StorageHasher, StorageMapKey};

use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct GuildFilter {
    pub name: GuildName,
    pub role: Option<RoleName>,
}

pub async fn registered_operators(api: Api) -> Result<Vec<AccountId>, subxt::Error> {
    let operators = runtime::storage().chainlink().operators();
    Ok(api
        .storage()
        .fetch(&operators, None)
        .await?
        .unwrap_or_default())
}

pub async fn members(
    api: Api,
    filter: Option<&GuildFilter>,
    page_size: u32,
) -> Result<Vec<AccountId>, subxt::Error> {
    let mut keys = Vec::new();
    if let Some(guild_filter) = filter {
        let role_ids = role_id(api.clone(), guild_filter, page_size).await?;
        for role_id in role_ids.into_iter() {
            let mut query_key = runtime::storage().guild().members_root().to_root_bytes();
            StorageMapKey::new(role_id, StorageHasher::Blake2_128).to_bytes(&mut query_key);
            let mut storage_keys = api
                .storage()
                .fetch_keys(&query_key, page_size, None, None)
                .await?;
            keys.append(&mut storage_keys);
        }
    } else {
        // read everything from root
        let query_key = runtime::storage().guild().members_root().to_root_bytes();
        let mut storage_keys = api
            .storage()
            .fetch_keys(&query_key, page_size, None, None)
            .await?;
        keys.append(&mut storage_keys);
    }

    Ok(keys
        .iter()
        .map(|key| AccountId::try_from(&key.0[96..128]).unwrap())
        .collect())
}

pub async fn guild_id(api: Api, name: GuildName) -> Result<Hash, subxt::Error> {
    let guild_id_address = runtime::storage().guild().guild_id_map(name);
    api.storage()
        .fetch(&guild_id_address, None)
        .await?
        .ok_or_else(|| subxt::Error::Other(format!("no such Guild registered: {:?}", name)))
}

pub async fn role_id(
    api: Api,
    filter: &GuildFilter,
    page_size: u32,
) -> Result<Vec<Hash>, subxt::Error> {
    let guild_id = guild_id(api.clone(), filter.name).await?;
    let mut query_key = runtime::storage()
        .guild()
        .role_id_map_root()
        .to_root_bytes();

    StorageMapKey::new(guild_id, StorageHasher::Blake2_128).to_bytes(&mut query_key);
    if let Some(rn) = filter.role {
        query_key.append(&mut guild_id.0.to_vec());
        StorageMapKey::new(rn, StorageHasher::Blake2_128).to_bytes(&mut query_key);
    }

    let keys = api
        .storage()
        .fetch_keys(&query_key, page_size, None, None)
        .await?;

    let mut role_ids = Vec::new();
    for key in keys.iter() {
        let role_id_bytes_vec = api
            .storage()
            .fetch_raw(&key.0, None)
            .await?
            .ok_or_else(|| subxt::Error::Other(format!("invalid key {:?}", key)))?;
        let role_id_bytes: [u8; 32] = role_id_bytes_vec
            .as_slice()
            .try_into()
            .map_err(|_| subxt::error::DecodeError::InvalidChar(9999))?;
        role_ids.push(role_id_bytes.into());
    }
    Ok(role_ids)
}

pub async fn join_request(api: Api, id: RequestIdentifier) -> Result<JoinRequest, subxt::Error> {
    let key = runtime::storage().guild().join_requests(id);
    let join_request = api
        .storage()
        .fetch(&key, None)
        .await?
        .ok_or_else(|| subxt::Error::Other(format!("no Guild join request with id: {}", id)))?;

    Ok(join_request)
}

pub async fn oracle_requests(
    api: Api,
    page_size: u32,
) -> Result<BTreeMap<u64, AccountId>, subxt::Error> {
    let root = runtime::storage().chainlink().requests_root();

    let mut map = BTreeMap::new();
    let mut iter = api.storage().iter(root, page_size, None).await?;
    while let Some((key, value)) = iter.next().await? {
        let key_bytes = (&key.0[48..]).try_into().unwrap();
        map.insert(u64::from_le_bytes(key_bytes), value.operator);
    }
    Ok(map)
}
