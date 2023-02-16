mod functions;
pub use functions::*;

use crate::{Hash, SubxtError};
use gn_common::Role;
use gn_engine::RequirementsWithLogic;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilteredRequirements {
    pub filter: Option<gn_common::filter::Filter<Hash>>,
    pub requirements: Option<gn_engine::RequirementsWithLogic>,
}

impl TryFrom<Role<Hash>> for FilteredRequirements {
    type Error = SubxtError;
    fn try_from(role: Role<Hash>) -> Result<Self, Self::Error> {
        let requirements = if let Some(serialized_requirements) = role.requirements {
            let reqs_with_logic =
                RequirementsWithLogic::from_serialized_tuple(serialized_requirements)
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
