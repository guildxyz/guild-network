pub mod guild;
pub mod identity;

use crate::{runtime, AccountId, MultiAddress, SessionKeys};
use subxt::dynamic::Value;
pub use subxt::tx::{DynamicTxPayload as TxPayload, TxPayload as TxPayloadT};

pub fn sudo<'a>(call: TxPayload<'_>) -> TxPayload<'a> {
    subxt::dynamic::tx("Sudo", "sudo", vec![("call", call.into_value())])
}

pub fn register_operator<'a>(operator: &AccountId) -> TxPayload<'a> {
    subxt::dynamic::tx(
        "Oracle",
        "register_operator",
        vec![("operator", Value::from_bytes(operator))],
    )
}

pub fn deregister_operator<'a>(operator: &AccountId) -> TxPayload<'a> {
    subxt::dynamic::tx(
        "Oracle",
        "deregister_operator",
        vec![("operator", Value::from_bytes(operator))],
    )
}

pub fn add_validator<'a>(validator: &AccountId) -> TxPayload<'a> {
    subxt::dynamic::tx(
        "ValidatorManager",
        "add_validator",
        vec![("validator_id", Value::from_bytes(validator))],
    )
}

pub fn remove_validator<'a>(validator: &AccountId) -> TxPayload<'a> {
    subxt::dynamic::tx(
        "ValidatorManager",
        "remove_validator",
        vec![("validator_id", Value::from_bytes(validator))],
    )
}

pub fn transfer(account: &AccountId, amount: u128) -> impl TxPayloadT {
    runtime::tx()
        .balances()
        .transfer(MultiAddress::Id(account.clone()), amount)
}

pub fn activate_operator() -> impl TxPayloadT {
    runtime::tx().oracle().activate_operator()
}

pub fn deactivate_operator() -> impl TxPayloadT {
    runtime::tx().oracle().deactivate_operator()
}

pub fn set_session_keys(keys: SessionKeys, proof: Vec<u8>) -> impl TxPayloadT {
    runtime::tx().session().set_keys(keys, proof)
}
