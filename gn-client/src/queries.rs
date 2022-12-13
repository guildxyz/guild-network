use crate::{cbor_deserialize, runtime, AccountId, Api, GuildData, Hash, Request, RuntimeIdentity};
use gn_common::identities::Identity;
use gn_common::requirements::RequirementsWithLogic;
use gn_common::{GuildName, RequestIdentifier, RoleName};
use subxt::ext::codec::Decode;
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

pub async fn user_identities(
    api: Api,
    page_size: u32,
) -> Result<BTreeMap<AccountId, Vec<Identity>>, subxt::Error> {
    let root = runtime::storage().guild().user_data_root();
    let mut map = BTreeMap::new();
    let mut iter = api.storage().iter(root, page_size, None).await?;
    while let Some((key, identities)) = iter.next().await? {
        let identities = identities
            .into_iter()
            // NOTE safety: these are the same types defined at two different
            // places by the subxt macro
            .map(|x| unsafe { std::mem::transmute::<RuntimeIdentity, Identity>(x) })
            .collect();
        // NOTE unwrap is fine because we are creating an account id from 32 bytes
        let account_id = AccountId::try_from(&key.0[48..80]).unwrap();
        map.insert(account_id, identities);
    }
    Ok(map)
}

pub async fn user_identity(api: Api, user_id: &AccountId) -> Result<Vec<Identity>, subxt::Error> {
    let key = runtime::storage().guild().user_data(user_id);
    let ids = api.storage().fetch(&key, None).await?.unwrap_or_default();
    Ok(ids
        .into_iter()
        .map(|x| unsafe { std::mem::transmute::<RuntimeIdentity, Identity>(x) })
        .collect())
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

    // NOTE unwrap is fine because we are creating an account id from 32 bytes
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
        .ok_or_else(|| subxt::Error::Other(format!("no such Guild registered: {name:?}")))
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
            .ok_or_else(|| subxt::Error::Other(format!("invalid key {key:?}")))?;
        let role_id_bytes: [u8; 32] = role_id_bytes_vec
            .as_slice()
            .try_into()
            .map_err(|_| subxt::error::DecodeError::InvalidChar(9999))?;
        role_ids.push(role_id_bytes.into());
    }
    Ok(role_ids)
}

pub async fn oracle_request(api: Api, id: RequestIdentifier) -> Result<Request, subxt::Error> {
    let key = runtime::storage().chainlink().requests(id);
    let request = api
        .storage()
        .fetch(&key, None)
        .await?
        .ok_or_else(|| subxt::Error::Other(format!("no request with id: {id}")))?;

    let request = Request::decode(&mut request.data.as_slice())?;

    Ok(request)
}

pub async fn oracle_requests(
    api: Api,
    page_size: u32,
) -> Result<BTreeMap<RequestIdentifier, AccountId>, subxt::Error> {
    let root = runtime::storage().chainlink().requests_root();

    let mut map = BTreeMap::new();
    let mut iter = api.storage().iter(root, page_size, None).await?;
    while let Some((key, value)) = iter.next().await? {
        let key_bytes = (&key.0[48..]).try_into().unwrap();
        map.insert(RequestIdentifier::from_le_bytes(key_bytes), value.operator);
    }
    Ok(map)
}

pub async fn guild(
    api: Api,
    filter: Option<GuildName>,
    page_size: u32,
) -> Result<BTreeMap<GuildName, GuildData>, subxt::Error> {
    let mut guilds_map = BTreeMap::new();
    if let Some(name) = filter {
        let guild_id = guild_id(api.clone(), name).await?;
        let guild_addr = runtime::storage().guild().guilds(guild_id);
        let guild = api
            .storage()
            .fetch(&guild_addr, None)
            .await?
            .ok_or_else(|| subxt::Error::Other(format!("no Guild with name: {name:#?}")))?;
        guilds_map.insert(guild.name, guild.data);
    } else {
        let root = runtime::storage().guild().guilds_root();
        let mut iter = api.storage().iter(root, page_size, None).await?;
        while let Some((_guild_uuid, guild_data)) = iter.next().await? {
            // we don't care about guild_uuid in this case
            guilds_map.insert(guild_data.name, guild_data.data);
        }
    }
    Ok(guilds_map)
}

pub async fn requirements(
    api: Api,
    guild_name: GuildName,
    role_name: RoleName,
) -> Result<RequirementsWithLogic, subxt::Error> {
    let filter = GuildFilter {
        name: guild_name,
        role: Some(role_name),
    };
    let role_id = role_id(api.clone(), &filter, 1).await?;
    let requirements_addr = runtime::storage().guild().roles(role_id[0]);
    let requirements_vec = api
        .storage()
        .fetch(&requirements_addr, None)
        .await?
        .ok_or_else(|| subxt::Error::Other(format!("no role with name: {role_name:#?}")))?;

    cbor_deserialize(&requirements_vec).map_err(|e| subxt::Error::Other(e.to_string()))
}
