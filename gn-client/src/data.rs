use crate::runtime::runtime_types::pallet_guild::pallet::Guild as RuntimeGuildData;
use crate::AccountId;
use gn_common::pad::unpad_from_32_bytes;
use gn_common::requirements::RequirementsWithLogic;
use gn_common::{GuildName, RoleName};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guild {
    pub name: GuildName,
    pub metadata: Vec<u8>,
    pub roles: Vec<Role>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: RoleName,
    pub reqs: RequirementsWithLogic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildData {
    pub name: String,
    pub owner: AccountId,
    pub metadata: Vec<u8>,
    pub roles: Vec<String>,
}

impl From<RuntimeGuildData<AccountId>> for GuildData {
    fn from(value: RuntimeGuildData<AccountId>) -> Self {
        Self {
            name: unpad_from_32_bytes(&value.name),
            owner: value.data.owner,
            metadata: value.data.metadata,
            roles: value
                .data
                .roles
                .into_iter()
                .map(|role| unpad_from_32_bytes(&role))
                .collect(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use gn_common::pad::pad_to_32_bytes;
    use gn_common::{EvmAddress, U256};

    fn address(num: u32) -> EvmAddress {
        let mut n = [0; 20];
        n[0..4].copy_from_slice(&num.to_le_bytes());
        n
    }

    fn u256(num: u32) -> U256 {
        let mut n = [0; 32];
        n[0..4].copy_from_slice(&num.to_le_bytes());
        n
    }

    #[test]
    fn serialized_tx_input() {
        use gn_common::requirements::allowlist::Allowlist;
        use gn_common::requirements::balance::*;
        use gn_common::requirements::chains::EvmChain;
        use gn_common::requirements::Requirement;

        let guild_name = "hellobello";
        let role_names = vec!["myrole", "mysecondrole", "mythirdrole"];

        let logics = vec!["0", "0 OR 1", "(0 AND 1) OR 2"];
        let requirements = vec![
            vec![Requirement::Free],
            vec![
                Requirement::EvmAllowlist(Allowlist::new(vec![
                    address(1),
                    address(2),
                    address(3),
                    address(4),
                    address(5),
                ])),
                Requirement::EvmAllowlist(Allowlist::new(vec![
                    address(9),
                    address(10),
                    address(11),
                ])),
            ],
            vec![
                Requirement::EvmBalance(RequiredBalance {
                    token_type: None,
                    relation: Relation::GreaterThan(u256(2)),
                    chain: EvmChain::Ethereum,
                }),
                Requirement::EvmBalance(RequiredBalance {
                    token_type: Some(TokenType::Fungible {
                        address: address(134),
                    }),
                    relation: Relation::Between(u256(10)..u256(20)),
                    chain: EvmChain::Bsc,
                }),
                Requirement::EvmBalance(RequiredBalance {
                    token_type: Some(TokenType::NonFungible {
                        address: address(1555),
                        id: u256(23),
                    }),
                    relation: Relation::EqualTo(u256(1)),
                    chain: EvmChain::Polygon,
                }),
            ],
        ];

        let roles = role_names
            .into_iter()
            .zip(logics.into_iter().zip(requirements.into_iter()))
            .map(|(name, (logic, requirements))| Role {
                name: pad_to_32_bytes(name),
                reqs: RequirementsWithLogic {
                    logic: logic.to_string(),
                    requirements,
                },
            })
            .collect();

        let guild = Guild {
            name: pad_to_32_bytes(guild_name),
            metadata: vec![1, 2, 3],
            roles,
        };

        let serialized = serde_json::to_string(&guild).unwrap();
        println!("{serialized}");
    }
}
