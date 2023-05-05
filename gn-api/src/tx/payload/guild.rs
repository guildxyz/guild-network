use super::*;
use crate::{cast, SubxtError};
use gn_common::filter::{Guild as GuildFilter, Logic as FilterLogic};
use gn_common::merkle::Proof as MerkleProof;
use gn_common::{GuildName, RoleName};
use guild_requirements::{RequirementsWithLogic, SerializedRequirementsWithLogic};

pub fn create_guild(guild_name: GuildName, metadata: Vec<u8>) -> impl TxPayloadT {
    runtime::tx().guild().create_guild(guild_name, metadata)
}

pub fn create_free_role(guild_name: GuildName, role_name: RoleName) -> impl TxPayloadT {
    runtime::tx()
        .guild()
        .create_free_role(guild_name, role_name)
}

pub fn create_role_with_allowlist(
    guild_name: GuildName,
    role_name: RoleName,
    allowlist: Vec<AccountId>,
    filter_logic: FilterLogic,
    requirements: Option<RequirementsWithLogic>,
) -> Result<impl TxPayloadT, SubxtError> {
    let serialized_requirements = requirements
        .map(SerializedRequirementsWithLogic::try_from)
        .transpose()
        .map_err(|e| SubxtError::Other(e.to_string()))?
        .map(|serialized| (serialized.requirements, serialized.logic));
    Ok(runtime::tx().guild().create_role_with_allowlist(
        guild_name,
        role_name,
        allowlist,
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
) -> Result<impl TxPayloadT, SubxtError> {
    let serialized_requirements = requirements
        .map(SerializedRequirementsWithLogic::try_from)
        .transpose()
        .map_err(|e| SubxtError::Other(e.to_string()))?
        .map(|serialized| (serialized.requirements, serialized.logic));
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
) -> Result<impl TxPayloadT, SubxtError> {
    let serialized_requirements = SerializedRequirementsWithLogic::try_from(requirements)
        .map(|serialized| (serialized.requirements, serialized.logic))
        .map_err(|e| SubxtError::Other(e.to_string()))?;
    Ok(runtime::tx()
        .guild()
        .create_unfiltered_role(guild_name, role_name, serialized_requirements))
}

pub fn leave(guild_name: GuildName, role_name: RoleName) -> impl TxPayloadT {
    runtime::tx().guild().leave(guild_name, role_name)
}

pub fn join_free_role(guild_name: GuildName, role_name: RoleName) -> impl TxPayloadT {
    runtime::tx().guild().join_free_role(guild_name, role_name)
}

pub fn join_child_role(guild_name: GuildName, role_name: RoleName) -> impl TxPayloadT {
    runtime::tx().guild().join_child_role(guild_name, role_name)
}

pub fn join_role_with_allowlist(
    guild_name: GuildName,
    role_name: RoleName,
    merkle_proof: MerkleProof,
) -> impl TxPayloadT {
    runtime::tx().guild().join_role_with_allowlist(
        guild_name,
        role_name,
        cast::proof::to_runtime(merkle_proof),
    )
}

pub fn join_unfiltered_role(guild_name: GuildName, role_name: RoleName) -> impl TxPayloadT {
    runtime::tx()
        .guild()
        .join_unfiltered_role(guild_name, role_name)
}

pub fn callback(request_id: u64, result: bool) -> impl TxPayloadT {
    runtime::tx().guild().callback(request_id, result)
}
