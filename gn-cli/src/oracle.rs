use futures::{future::try_join_all, StreamExt};
use gn_api::{
    query,
    tx::{self, Signer},
    Api, GuildCall, OracleCallback, OracleRequest, SubxtError,
};
use gn_common::identity::Identity;
use gn_common::utils::{matches_variant, verification_msg};
use gn_common::{RequestData, RequestIdentifier};

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

pub async fn oracle(api: Api, operator: Arc<Signer>, activate: bool) {
    if !query::is_operator_registered(api.clone(), operator.account_id())
        .await
        .expect("failed to fetch operator info")
    {
        panic!("{} is not registered as an operator", operator.account_id());
    }

    if activate {
        tx::send::in_block(api.clone(), &tx::activate_operator(), Arc::clone(&operator))
            .await
            .expect("failed to activate operator");

        log::info!("operator activation request submitted");
    }

    let active = query::active_operators(api.clone())
        .await
        .expect("failed to fetch active operators");
    if !active.contains(operator.account_id()) {
        panic!(
            "{} not activated. Run oracle with the '--activate' flag",
            operator.account_id()
        );
    } else {
        log::info!("node activated, listening to events...");
    }

    let mut subscription = api
        .blocks()
        .subscribe_best()
        .await
        .expect("failed to subscribe to blocks");

    while let Some(block_result) = subscription.next().await {
        match block_result {
            Ok(block) => match block.events().await {
                Ok(events) => {
                    let requests = events
                        .iter()
                        .filter_map(|event_result| event_result.ok())
                        .filter_map(|event_details| {
                            event_details.as_event::<OracleRequest>().ok().flatten()
                        })
                        .collect::<Vec<OracleRequest>>();
                    submit_answers(api.clone(), Arc::clone(&operator), requests)
                }
                Err(err) => log::error!("invalid block events: {err}"),
            },
            Err(err) => log::error!("invalid block: {err}"),
        }
    }
    log::error!("block subscription aborted");
}

fn submit_answers(api: Api, signer: Arc<Signer>, requests: Vec<OracleRequest>) {
    tokio::spawn(async move {
        let answer_futures = requests
            .into_iter()
            .filter(|request| {
                if &request.operator != signer.account_id() {
                    // request wasn't delegated to us so return
                    log::trace!("request not delegated to us");
                    return false;
                }

                // check whether the incoming request originates from the guild
                // pallet just for testing basically
                if !matches_variant(&request.callback, &GuildCall::callback { result: vec![] }) {
                    log::trace!("callback mismatch");
                    return false;
                }
                true
            })
            .map(|request| {
                log::info!(
                    "OracleRequest: {}, {}, {:?}, {}",
                    request.request_id,
                    request.operator,
                    request.callback,
                    request.fee
                );

                compile_answer(api.clone(), request.request_id)
            })
            .collect::<Vec<_>>();

        match try_join_all(answer_futures).await {
            Ok(answers) => {
                if let Err(e) = tx::send::batch(api, answers.iter(), signer).await {
                    log::warn!("failed to send oracle answers: {}", e)
                }
            }
            Err(e) => log::warn!("failed to compile oracle answers: {}", e),
        }
    });
}

async fn compile_answer(
    api: Api,
    request_id: RequestIdentifier,
) -> Result<OracleCallback, SubxtError> {
    let oracle_request = query::oracle_request(api.clone(), request_id).await?;

    let oracle_answer = match oracle_request.data {
        RequestData::Register {
            identity_with_auth,
            index: _,
        } => {
            log::info!("[registration request] acc: {}", oracle_request.requester);
            // deserialize user identities
            let expected_msg = verification_msg(&oracle_request.requester);
            identity_with_auth.verify(expected_msg)
        }
        RequestData::ReqCheck {
            account,
            guild_name,
            role_name,
        } => {
            log::info!(
                "[requirement check request] acc: {}, guild: {:?}, role: {:?}",
                account,
                guild_name,
                role_name,
            );
            // fetch requirements
            let requirements_with_logic = query::requirements(api.clone(), guild_name, role_name)
                .await?
                .ok_or(SubxtError::Other("no requirements found".to_string()))?;
            // build requireemnt tree from logic
            let requirement_tree = requiem::LogicTree::from_str(&requirements_with_logic.logic)
                .map_err(|e| SubxtError::Other(e.to_string()))?;
            let identities = query::user_identity(api.clone(), &account).await?;
            let maybe_address = identities
                .iter()
                .find(|&x| matches_variant(x, &Identity::Address20([0u8; 20])));

            if let Some(address) = maybe_address {
                let requirement_futures = requirements_with_logic
                    .requirements
                    .iter()
                    .map(|req| req.check(address))
                    .collect::<Vec<_>>();
                match futures::future::try_join_all(requirement_futures).await {
                    Ok(boolean_vec) => {
                        let requirement_check_map: HashMap<u32, bool> = boolean_vec
                            .into_iter()
                            .enumerate()
                            .map(|(i, b)| (i as u32, b))
                            .collect();
                        requirement_tree
                            .evaluate(&requirement_check_map)
                            .unwrap_or(false)
                    }
                    Err(error) => {
                        log::warn!("requirement check failed: {}", error);
                        false
                    }
                }
            } else {
                log::warn!("requirement check failed: no registered evm identity");
                false
            }
        }
    };

    let result = vec![u8::from(oracle_answer)];
    log::info!("oracle answer ({}): {:?}", request_id, result);
    Ok(tx::oracle_callback(request_id, result))
}
