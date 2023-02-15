use super::FilteredRequirements;
use crate::{cast, runtime, AccountId, Api, Hash, Request, SubxtError};
use gn_common::filter::Guild as GuildFilter;
use gn_common::identity::Identity;
use gn_common::{Guild, GuildName, RequestIdentifier, RoleName};
use gn_engine::RequirementsWithLogic;
use subxt::ext::codec::Decode;
use subxt::storage::address::{StorageHasher, StorageMapKey};

use std::collections::BTreeMap;

pub async fn registered_operators(api: Api) -> Result<Vec<AccountId>, SubxtError> {
    let operators = runtime::storage().oracle().operators();
    Ok(api
        .storage()
        .at(None)
        .await?
        .fetch(&operators)
        .await?
        .unwrap_or_default())
}

pub async fn user_identity(api: Api, user_id: &AccountId) -> Result<Vec<Identity>, SubxtError> {
    let mut i = 0u8;
    let mut identities = Vec::new();
    while let Some(identity) = api
        .storage()
        .at(None)
        .await?
        .fetch(&runtime::storage().guild().user_data(user_id, i))
        .await?
    {
        identities.push(cast::id::from_runtime(identity));
        i += 1;
    }
    Ok(identities)
}

pub async fn members(
    api: Api,
    filter: Option<&GuildFilter>,
    page_size: u32,
) -> Result<Vec<AccountId>, SubxtError> {
    let mut keys = Vec::new();
    if let Some(guild_filter) = filter {
        let role_ids = role_id(api.clone(), guild_filter, page_size).await?;
        for role_id in role_ids.into_iter() {
            let mut query_key = runtime::storage().guild().members_root().to_root_bytes();
            StorageMapKey::new(role_id, StorageHasher::Blake2_128).to_bytes(&mut query_key);
            let mut storage_keys = api
                .storage()
                .at(None)
                .await?
                .fetch_keys(&query_key, page_size, None)
                .await?;
            keys.append(&mut storage_keys);
        }
    } else {
        // read everything from root
        let query_key = runtime::storage().guild().members_root().to_root_bytes();
        let mut storage_keys = api
            .storage()
            .at(None)
            .await?
            .fetch_keys(&query_key, page_size, None)
            .await?;
        keys.append(&mut storage_keys);
    }

    // NOTE unwrap is fine because we are creating an account id from 32 bytes
    Ok(keys
        .iter()
        .map(|key| {
            let id: [u8; 32] = key.0[96..128].try_into().unwrap();
            AccountId::from(id)
        })
        .collect())
}

pub async fn guild_id(api: Api, name: GuildName) -> Result<Hash, SubxtError> {
    let guild_id_address = runtime::storage().guild().guild_id_map(name);
    api.storage()
        .at(None)
        .await?
        .fetch(&guild_id_address)
        .await?
        .ok_or_else(|| SubxtError::Other(format!("no such Guild registered: {name:?}")))
}

pub async fn role_id(
    api: Api,
    filter: &GuildFilter,
    page_size: u32,
) -> Result<Vec<Hash>, SubxtError> {
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
        .at(None)
        .await?
        .fetch_keys(&query_key, page_size, None)
        .await?;

    let mut role_ids = Vec::new();
    for key in keys.iter() {
        let role_id_bytes_vec = api
            .storage()
            .at(None)
            .await?
            .fetch_raw(&key.0)
            .await?
            .ok_or_else(|| SubxtError::Other(format!("invalid key {key:?}")))?;
        let role_id_bytes: [u8; 32] = role_id_bytes_vec
            .as_slice()
            .try_into()
            .map_err(|_| SubxtError::Other("failed to decode bytes".to_string()))?;
        role_ids.push(role_id_bytes.into());
    }
    Ok(role_ids)
}

pub async fn oracle_request(api: Api, id: RequestIdentifier) -> Result<Request, SubxtError> {
    let key = runtime::storage().oracle().requests(id);
    let request = api
        .storage()
        .at(None)
        .await?
        .fetch(&key)
        .await?
        .ok_or_else(|| SubxtError::Other(format!("no request with id: {id}")))?;

    let request = Request::decode(&mut request.data.as_slice())?;

    Ok(request)
}

pub async fn oracle_requests(
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

pub async fn guilds(
    api: Api,
    filter: Option<GuildName>,
    page_size: u32,
) -> Result<Vec<Guild<AccountId>>, SubxtError> {
    let mut guilds = Vec::new();
    if let Some(name) = filter {
        let guild_id = guild_id(api.clone(), name).await?;
        let guild_addr = runtime::storage().guild().guilds(guild_id);
        let guild = api
            .storage()
            .at(None)
            .await?
            .fetch(&guild_addr)
            .await?
            .ok_or_else(|| SubxtError::Other(format!("no Guild with name: {name:#?}")))?;
        guilds.push(cast::guild::from_runtime(guild));
    } else {
        let root = runtime::storage().guild().guilds_root();
        let mut iter = api.storage().at(None).await?.iter(root, page_size).await?;
        while let Some((_guild_uuid, guild)) = iter.next().await? {
            // we don't care about guild_uuid in this case
            guilds.push(cast::guild::from_runtime(guild));
        }
    }
    Ok(guilds)
}

pub async fn filtered_requirements(
    api: Api,
    guild_name: GuildName,
    role_name: RoleName,
) -> Result<FilteredRequirements, SubxtError> {
    let filter = GuildFilter {
        name: guild_name,
        role: Some(role_name),
    };
    let role_ids = role_id(api.clone(), &filter, 1).await?;
    let role_id = role_ids
        .get(0)
        .ok_or_else(|| SubxtError::Other(format!("no role with name: {role_name:#?}")))?;
    let role_key = runtime::storage().guild().roles(role_id);
    let role = api
        .storage()
        .at(None)
        .await?
        .fetch(&role_key)
        .await?
        .ok_or_else(|| SubxtError::Other(format!("no role with name: {role_name:#?}")))?;

    FilteredRequirements::try_from(cast::role::from_runtime(role))
}

pub async fn requirements(
    api: Api,
    guild_name: GuildName,
    role_name: RoleName,
) -> Result<Option<RequirementsWithLogic>, SubxtError> {
    filtered_requirements(api, guild_name, role_name)
        .await
        .map(|x| x.requirements)
}
