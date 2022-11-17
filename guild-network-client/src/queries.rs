use crate::{runtime, AccountId, Api, Hash};
use guild_network_common::{GuildName, RoleName};
use subxt::storage::address::{StorageHasher, StorageMapKey};

use std::collections::BTreeMap;

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

/*
pub async fn is_member(
    api: Api,
    guild_name: [u8; 32],
    role_name: [u8; 32],
    account: &AccountId,
) -> Result<(), subxt::Error> {
    let member = runtime::storage()
        .guild()
        .members(guild_name, role_name, account);
    api.storage()
        .fetch(&member, None)
        .await?
        .map(|_| ()) // turn Some(bool) into Some(())
        .ok_or_else(|| subxt::Error::Other("not a member".into())) // turn Some(()) to Ok(()) and None to Err(..)
}
*/

pub async fn members(
    api: Api,
    //filter: Option<GuildFilter>,
    page_size: u32,
) -> Result<BTreeMap<Hash, AccountId>, subxt::Error> {
    let members_root = runtime::storage().guild().members_root();

    let mut members_iter = api.storage().iter(members_root, page_size, None).await?;
    let mut members = BTreeMap::new();
    while let Some((key, _value)) = members_iter.next().await? {
        // NOTE unwrap is fine because the input length is 32
        let role_id_bytes: [u8; 32] = key.0[64..96].try_into().unwrap();
        let role_id = Hash::from(role_id_bytes);
        // NOTE unwrap is fine because the input length is 32
        let account_id = AccountId::try_from(&key.0[96..128]).unwrap();
        members.insert(role_id, account_id);
    }
    Ok(members)
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
    guild_name: GuildName,
    role_name: Option<RoleName>,
    page_size: u32,
) -> Result<Vec<Hash>, subxt::Error> {
    let guild_id = guild_id(api.clone(), guild_name).await?;
    let mut query_key = runtime::storage()
        .guild()
        .role_id_map_root()
        .to_root_bytes();
    StorageMapKey::new(guild_id, StorageHasher::Blake2_128).to_bytes(&mut query_key);
    if let Some(rn) = role_name {
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

pub async fn join_requests(api: Api, page_size: u32) -> Result<(), subxt::Error> {
    let root = runtime::storage().guild().join_requests_root();

    let mut iter = api.storage().iter(root, page_size, None).await?;
    while let Some((key, value)) = iter.next().await? {
        println!("key: {:?}\tvalue: {:?}", key, value);
    }
    Ok(())
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
