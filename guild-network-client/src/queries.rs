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

pub async fn members(
    api: Api,
    guild_name: Option<[u8; 32]>,
    role_name: Option<[u8; 32]>,
) -> Result<Vec<AccountId>, subxt::Error> {
    let members_root = runtime::storage().guild().members_root();

    let page_size = 10;
    let mut members_iter = api.storage().iter(members_root, page_size, None).await?;
    let mut members_vec = Vec::with_capacity(page_size as usize);
    while let Some(((guild, role, account), _)) = members_iter.next().await? {
        match (guild_name, role_name) {
            (Some(gn), Some(rn)) => {
                if gn == guild && rn == role {
                    members_vec.push(account);
                }
            }
            (Some(gn), None) => {
                if gn == guild {
                    members_vec.push(account);
                }
            }
            (None, None) => members_vec.push(account),
            _ => {}
        }
    }
    Ok(members_vec)
}
