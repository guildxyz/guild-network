pub mod guild;
pub mod identity;
pub mod oracle;

use crate::{runtime, AccountId, Api, SessionKeys, SubxtError};
use gn_common::Role;
use guild_requirements::{RequirementsWithLogic, SerializedRequirementsWithLogic};

#[derive(Clone, Debug)]
pub struct FilteredRequirements {
    pub filter: Option<gn_common::filter::Filter>,
    pub requirements: Option<guild_requirements::RequirementsWithLogic>,
}

impl TryFrom<Role> for FilteredRequirements {
    type Error = SubxtError;
    fn try_from(role: Role) -> Result<Self, Self::Error> {
        let requirements = if let Some(serialized_requirements) = role.requirements {
            let serialized_requirements = SerializedRequirementsWithLogic {
                requirements: serialized_requirements.0,
                logic: serialized_requirements.1,
            };
            let reqs_with_logic = RequirementsWithLogic::try_from(serialized_requirements)
                .map_err(|err| SubxtError::Other(err.to_string()))?;
            Some(reqs_with_logic)
        } else {
            None
        };

        Ok(Self {
            filter: role.filter,
            requirements,
        })
    }
}

pub async fn is_validator_added(api: Api, id: &AccountId) -> Result<bool, SubxtError> {
    let validators_key = runtime::storage().validator_manager().validators();
    let validators = api
        .storage()
        .at(None)
        .await?
        .fetch(&validators_key)
        .await?
        .ok_or(SubxtError::Other("empty validator set".to_string()))?;

    Ok(validators.contains(id))
}

pub async fn next_session_keys(api: Api, validator: &AccountId) -> Result<SessionKeys, SubxtError> {
    let storage_key = runtime::storage().session().next_keys(validator);
    let keys = api
        .storage()
        .at(None)
        .await?
        .fetch(&storage_key)
        .await?
        .ok_or_else(|| SubxtError::Other(format!("no session key set for: {validator}")))?;

    Ok(keys)
}
