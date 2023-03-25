use crate::{cast, runtime, AccountId, MultiAddress, SessionKeys, SubxtError};
use gn_common::filter::{Guild as GuildFilter, Logic as FilterLogic};
use gn_common::identity::{Identity, IdentityWithAuth};
use gn_common::merkle::Proof as MerkleProof;
use gn_common::{GuildName, RoleName};
use gn_engine::RequirementsWithLogic;
use subxt::dynamic::Value;
use subxt::tx::{DynamicTxPayload, TxPayload};

pub fn sudo<'a>(call: DynamicTxPayload<'_>) -> DynamicTxPayload<'a> {
    subxt::dynamic::tx("Sudo", "sudo", vec![("call", call.into_value())])
}

pub fn register_operator<'a>(operator: &AccountId) -> DynamicTxPayload<'a> {
    subxt::dynamic::tx(
        "Oracle",
        "register_operator",
        vec![("operator", Value::from_bytes(operator))],
    )
}

pub fn deregister_operator<'a>(operator: &AccountId) -> DynamicTxPayload<'a> {
    subxt::dynamic::tx(
        "Oracle",
        "deregister_operator",
        vec![("operator", Value::from_bytes(operator))],
    )
}

pub fn add_validator<'a>(validator: &AccountId) -> DynamicTxPayload<'a> {
    subxt::dynamic::tx(
        "ValidatorManager",
        "add_validator",
        vec![("validator_id", Value::from_bytes(validator))],
    )
}

pub fn remove_validator<'a>(validator: &AccountId) -> DynamicTxPayload<'a> {
    subxt::dynamic::tx(
        "ValidatorManager",
        "remove_validator",
        vec![("validator_id", Value::from_bytes(validator))],
    )
}

pub fn transfer(account: &AccountId, amount: u128) -> impl TxPayload {
    runtime::tx()
        .balances()
        .transfer(MultiAddress::Id(account.clone()), amount)
}

pub fn activate_operator() -> impl TxPayload {
    runtime::tx().oracle().activate_operator()
}

pub fn deactivate_operator() -> impl TxPayload {
    runtime::tx().oracle().deactivate_operator()
}

pub fn oracle_callback(request_id: u64, data: Vec<u8>) -> impl TxPayload {
    runtime::tx().oracle().callback(request_id, data)
}

pub fn create_guild(guild_name: GuildName, metadata: Vec<u8>) -> impl TxPayload {
    runtime::tx().guild().create_guild(guild_name, metadata)
}

pub fn create_free_role(guild_name: GuildName, role_name: RoleName) -> impl TxPayload {
    runtime::tx()
        .guild()
        .create_free_role(guild_name, role_name)
}

pub fn create_role_with_allowlist(
    guild_name: GuildName,
    role_name: RoleName,
    allowlist: Vec<Identity>,
    filter_logic: FilterLogic,
    requirements: Option<RequirementsWithLogic>,
) -> Result<impl TxPayload, SubxtError> {
    let serialized_requirements = requirements
        .map(RequirementsWithLogic::into_serialized_tuple)
        .transpose()
        .map_err(|e| SubxtError::Other(e.to_string()))?;
    Ok(runtime::tx().guild().create_role_with_allowlist(
        guild_name,
        role_name,
        cast::id_vec::to_runtime(allowlist),
        cast::filter_logic::to_runtime(filter_logic),
        serialized_requirements,
    ))
}

pub fn create_child_role(
    guild_name: GuildName,
    role_name: RoleName,
    filter: GuildFilter,
    filter_logic: FilterLogic,
    requirements: Option<RequirementsWithLogic>,
) -> Result<impl TxPayload, SubxtError> {
    let serialized_requirements = requirements
        .map(RequirementsWithLogic::into_serialized_tuple)
        .transpose()
        .map_err(|e| SubxtError::Other(e.to_string()))?;
    Ok(runtime::tx().guild().create_child_role(
        guild_name,
        role_name,
        cast::guild_filter::to_runtime(filter),
        cast::filter_logic::to_runtime(filter_logic),
        serialized_requirements,
    ))
}

pub fn create_unfiltered_role(
    guild_name: GuildName,
    role_name: RoleName,
    requirements: RequirementsWithLogic,
) -> Result<impl TxPayload, SubxtError> {
    let serialized_requirements = requirements
        .into_serialized_tuple()
        .map_err(|e| SubxtError::Other(e.to_string()))?;
    Ok(runtime::tx()
        .guild()
        .create_unfiltered_role(guild_name, role_name, serialized_requirements))
}

pub fn register(identity_with_auth: IdentityWithAuth, index: u8) -> impl TxPayload {
    runtime::tx()
        .guild()
        .register(cast::id_with_auth::to_runtime(identity_with_auth), index)
}

pub fn join(
    guild_name: GuildName,
    role_name: RoleName,
    proof: Option<MerkleProof>,
) -> impl TxPayload {
    runtime::tx()
        .guild()
        .join(guild_name, role_name, proof.map(cast::proof::to_runtime))
}

pub fn leave(guild_name: GuildName, role_name: RoleName) -> impl TxPayload {
    runtime::tx().guild().leave(guild_name, role_name)
}

pub fn set_session_keys(keys: SessionKeys, proof: Vec<u8>) -> impl TxPayload {
    runtime::tx().session().set_keys(keys, proof)
}
