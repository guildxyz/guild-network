#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

use gn_client::{query, AccountId, Api};
use gn_common::filter::Guild as GuildFilter;
use gn_common::identity::Identity;
use gn_common::merkle::Proof;
use gn_common::SerializedRequirements;
use gn_common::{pad::pad_to_n_bytes, GuildName, RoleName};
use gn_engine::RequirementsWithLogic;
use serde_wasm_bindgen::{from_value as deserialize_from_value, to_value as serialize_to_value};
use wasm_bindgen::prelude::*;

use std::str::FromStr;

const PAD_BYTES: usize = 32;

fn sanitize_name(name: String) -> Result<[u8; PAD_BYTES], JsValue> {
    if name.is_empty() || name.len() > PAD_BYTES {
        return Err(JsValue::from("invalid name length"));
    }
    Ok(pad_to_n_bytes::<PAD_BYTES, _>(&name))
}

#[wasm_bindgen(js_name = "queryMembers")]
pub async fn query_members(
    guild: String,
    role: Option<String>,
    url: String,
) -> Result<JsValue, JsValue> {
    let api = Api::from_url(&url)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    let guild_name = sanitize_name(guild)?;
    let role_name: Option<RoleName> = role.map(sanitize_name).transpose()?;
    let filter = GuildFilter {
        name: guild_name,
        role: role_name,
    };

    let members = query::members(api, &filter, 10)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    serialize_to_value(&members).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen(js_name = "queryGuilds")]
pub async fn query_guilds(guild: Option<String>, url: String) -> Result<JsValue, JsValue> {
    let api = Api::from_url(&url)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    let guild_name: Option<GuildName> = guild.map(sanitize_name).transpose()?;

    let guilds = query::guilds(api, guild_name, 10)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    serialize_to_value(&guilds).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen(js_name = "queryRequirements")]
pub async fn query_requirements(
    guild: String,
    role: String,
    url: String,
) -> Result<JsValue, JsValue> {
    let api = Api::from_url(&url)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    let guild_name = sanitize_name(guild)?;
    let role_name = sanitize_name(role)?;

    let requirements = query::filtered_requirements(api, guild_name, role_name)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;
    serialize_to_value(&requirements).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen(js_name = "queryUserIdentity")]
pub async fn query_user_identity(address: String, url: String) -> Result<JsValue, JsValue> {
    let id = AccountId::from_str(&address).map_err(|e| JsValue::from(e.to_string()))?;
    let api = Api::from_url(&url)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    let identities = query::user_identity(api, &id)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    serialize_to_value(&identities).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen(js_name = "queryAllowlist")]
pub async fn query_allowlist(guild: String, role: String, url: String) -> Result<JsValue, JsValue> {
    let api = Api::from_url(&url)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    let guild_name = sanitize_name(guild)?;
    let role_name = sanitize_name(role)?;

    let allowlist = query::allowlist(api, guild_name, role_name)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;
    serialize_to_value(&allowlist).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen(js_name = "generateMerkleProof")]
pub fn generate_merkle_proof(
    list: JsValue,
    leaf_index: usize,
    id_index: u8,
) -> Result<JsValue, JsValue> {
    let allowlist: Vec<Identity> =
        deserialize_from_value(list).map_err(|e| JsValue::from(e.to_string()))?;
    serialize_to_value(&Proof::new(&allowlist, leaf_index, id_index))
        .map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen(js_name = "verificationMsg")]
pub fn verification_msg(address: String) -> Result<String, JsValue> {
    let account_id = AccountId::from_str(&address).map_err(|e| JsValue::from(e.to_string()))?;
    Ok(gn_common::utils::verification_msg(account_id))
}

#[wasm_bindgen(js_name = "serializeRequirements")]
pub fn serialize_requirements(requirements: JsValue) -> Result<JsValue, String> {
    let req = deserialize_from_value::<RequirementsWithLogic>(requirements)
        .map_err(|error| error.to_string())?;

    let req = req
        .into_serialized_tuple()
        .map_err(|error| error.to_string())?;

    serialize_to_value(&req).map_err(|error| error.to_string())
}

#[wasm_bindgen(js_name = "deserializeRequirements")]
pub fn deserialize_requirements(requirements: JsValue) -> Result<JsValue, String> {
    let req = deserialize_from_value::<SerializedRequirements>(requirements)
        .map_err(|error| error.to_string())?;

    let req =
        RequirementsWithLogic::from_serialized_tuple(req).map_err(|error| error.to_string())?;

    serialize_to_value(&req).map_err(|error| error.to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    use gn_engine::balance::{Balance, Relation, TokenType};
    use gn_engine::chains::EvmChain;
    use gn_engine::{EvmAddress, Requirement, U256};
    use gn_test_data as _;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn init_tracing() {
        console_error_panic_hook::set_once();
        tracing_wasm::set_as_global_default();
    }

    #[wasm_bindgen_test]
    async fn test_verification_msg_wrapper() {
        init_tracing();

        let account_id_str = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
        let account_id = AccountId::from_str(account_id_str).unwrap();
        assert_eq!(account_id.to_string(), account_id_str);

        let expected_msg = "Guild Network registration id: d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
        let msg = verification_msg(account_id_str.to_string()).unwrap();
        assert_eq!(msg, expected_msg);
    }

    #[wasm_bindgen_test]
    async fn test_serialization_roundtrip() {
        let tokens: Vec<Option<TokenType<EvmAddress, U256>>> = vec![
            None,
            Some(TokenType::Fungible {
                address: [99u8; 20],
            }),
            Some(TokenType::NonFungible {
                address: [0u8; 20],
                id: None,
            }),
            Some(TokenType::NonFungible {
                address: [1u8; 20],
                id: Some([223u8; 32]),
            }),
        ];

        let relations = vec![
            Relation::EqualTo([11u8; 32]),
            Relation::GreaterThan([12u8; 32]),
            Relation::LessOrEqualTo([13u8; 32]),
            Relation::Between([50u8; 32]..[100u8; 32]),
        ];

        let logic = "(0 AND 2) OR (1 AND 3)".to_string();

        let requirements = tokens
            .into_iter()
            .zip(relations.into_iter())
            .map(|(token_type, relation)| {
                Requirement::EvmBalance(Balance {
                    token_type,
                    relation,
                    chain: EvmChain::Ethereum,
                })
            })
            .collect();

        let requirements_with_logic = RequirementsWithLogic {
            requirements,
            logic,
        };

        let js_requirements_expected = serialize_to_value(&requirements_with_logic)
            .map_err(|error| error.to_string())
            .unwrap();

        let js_requirements_serialized =
            serialize_requirements(js_requirements_expected.clone()).unwrap();
        let js_requirements = deserialize_requirements(js_requirements_serialized).unwrap();

        assert_eq!(
            js_requirements.as_string(),
            js_requirements_expected.as_string()
        );

        let requirements_from_js: RequirementsWithLogic =
            deserialize_from_value(js_requirements).unwrap();

        assert_eq!(requirements_with_logic.logic, requirements_from_js.logic);
    }

    // NOTE these only work after the guild/join example
    // was successfully run
    #[cfg(feature = "queries")]
    mod queries {
        use super::*;
        use gn_client::{query::FilteredRequirements, AccountId};
        use gn_common::filter::{Filter, Logic as FilterLogic};
        use gn_common::identity::Identity;
        use gn_common::Guild;
        use gn_test_data::*;

        #[wasm_bindgen_test]
        async fn test_query_chain() {
            let api = Api::from_url(URL).await.unwrap();

            let chain = api.rpc().system_chain().await.unwrap();

            assert_eq!(chain, "Development");
        }

        #[wasm_bindgen_test]
        async fn test_query_members() {
            let guild = "myguild".to_string();
            let role = None;
            let members_js = query_members(guild, role, URL.to_string()).await.unwrap();
            let members_vec: Vec<AccountId> = deserialize_from_value(members_js).unwrap();

            assert_eq!(members_vec.len(), N_TEST_ACCOUNTS);

            let guild = "mysecondguild".to_string();
            let role = Some("myrole".to_string());
            let members_js = query_members(guild, role, URL.to_string()).await.unwrap();
            let members_vec: Vec<AccountId> = deserialize_from_value(members_js).unwrap();

            assert_eq!(members_vec.len(), N_TEST_ACCOUNTS / 2);
        }

        #[wasm_bindgen_test]
        async fn test_query_guilds() {
            let guilds_js = query_guilds(None, URL.to_string()).await.unwrap();
            let guilds: Vec<Guild<AccountId>> = deserialize_from_value(guilds_js).unwrap();

            assert!(guilds.len() == 2);
            for guild in &guilds {
                assert_eq!(guild.roles[0], pad_to_n_bytes::<PAD_BYTES, _>("myrole"));
                assert_eq!(
                    guild.roles[1],
                    pad_to_n_bytes::<PAD_BYTES, _>("mysecondrole")
                );
            }
        }

        #[wasm_bindgen_test]
        async fn test_query_requirements() {
            let guild_name = "myguild".to_string();
            let role_name = "myrole".to_string();
            let requirements_js = query_requirements(guild_name, role_name, URL.to_string())
                .await
                .unwrap();
            let requirements: FilteredRequirements =
                deserialize_from_value(requirements_js).unwrap();
            assert!(requirements.filter.is_none());
            assert!(requirements.requirements.is_none());

            let guild_name = "myguild".to_string();
            let role_name = "mysecondrole".to_string();
            let requirements_js =
                query_requirements(guild_name.clone(), role_name.clone(), URL.to_string())
                    .await
                    .unwrap();
            let requirements: FilteredRequirements =
                deserialize_from_value(requirements_js).unwrap();
            let root = gn_client::H256::from_str(
                "0xf6bace20645fc288795dc16cf6780d755772ba7fbe8815d78d911023ff3c8f5b",
            )
            .unwrap();
            assert_eq!(
                requirements.filter,
                Some(Filter::Allowlist(
                    root.0,
                    FilterLogic::And,
                    N_TEST_ACCOUNTS as u32
                ))
            );
            assert!(requirements.requirements.is_none());

            let allowlist_js = query_allowlist(guild_name, role_name, URL.to_string())
                .await
                .unwrap();
            let allowlist = deserialize_from_value::<Option<Vec<Identity>>>(allowlist_js)
                .unwrap()
                .unwrap();
            assert_eq!(allowlist.len(), N_TEST_ACCOUNTS);
        }

        #[wasm_bindgen_test]
        async fn test_query_user_identity() {
            use gn_common::utils::matches_variant;
            let account_id = AccountId::from_str(TEST_ADDRESS).unwrap();

            let members_js = query_members("myguild".to_string(), None, URL.to_string())
                .await
                .unwrap();
            let members_vec: Vec<AccountId> = deserialize_from_value(members_js).unwrap();
            assert!(members_vec.contains(&account_id));

            let identities_js = query_user_identity(TEST_ADDRESS.to_string(), URL.to_string())
                .await
                .unwrap();

            let identities: Vec<Identity> = deserialize_from_value(identities_js).unwrap();

            assert_eq!(identities.len(), 2);
            assert!(matches_variant(
                identities.get(0).unwrap(),
                &Identity::Address20([0u8; 20])
            ));
            assert!(matches_variant(
                identities.get(1).unwrap(),
                &Identity::Other([0u8; 64])
            ));
        }

        #[wasm_bindgen_test]
        async fn test_generate_proof() {
            let guild_name = "myguild".to_string();
            let role_name = "mysecondrole".to_string();
            let allowlist_js = query_allowlist(guild_name, role_name, URL.to_string())
                .await
                .unwrap();
            let proof_js = generate_merkle_proof(allowlist_js, 7, 0).unwrap();
            let proof: Proof = deserialize_from_value(proof_js).unwrap();
            assert_eq!(proof.path.len(), 4);
            assert_eq!(proof.id_index, 0);
        }
    }
}
