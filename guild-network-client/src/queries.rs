use crate::{runtime, AccountId, Api};

pub async fn registered_operators(api: Api) -> Result<Vec<AccountId>, subxt::Error> {
    let operators = runtime::storage().chainlink().operators();
    Ok(api
        .storage()
        .fetch(&operators, None)
        .await?
        .unwrap_or_default())
}

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

pub async fn members(api: Api, page_size: u32) -> Result<Vec<bool>, subxt::Error> {
    let members_root = runtime::storage().guild().members_root();

    let mut members_iter = api.storage().iter(members_root, page_size, None).await?;
    let mut members_vec = Vec::with_capacity(page_size as usize);
    while let Some((key, value)) = members_iter.next().await? {
        println!("key: {:?}\tvalue: {:?}", key, value);
        members_vec.push(value);
    }
    Ok(members_vec)
}

pub async fn join_requests(api: Api, page_size: u32) -> Result<(), subxt::Error> {
    let root = runtime::storage().guild().join_requests_root();

    let mut iter = api.storage().iter(root, page_size, None).await?;
    while let Some((key, value)) = iter.next().await? {
        println!("key: {:?}\tvalue: {:?}", key, value);
    }
    Ok(())
}

pub async fn oracle_requests(api: Api, page_size: u32) -> Result<(), subxt::Error> {
    let root = runtime::storage().chainlink().requests_root();

    let mut iter = api.storage().iter(root, page_size, None).await?;
    while let Some((key, value)) = iter.next().await? {
        println!("key: {:?}\tvalue: {:?}", key, value);
    }
    Ok(())
}
