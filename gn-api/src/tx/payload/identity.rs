use super::*;
use gn_common::{Authority, Identity, Prefix};

pub fn register() -> impl TxPayloadT {
    runtime::tx().guild_identity().register()
}

pub fn deregister() -> impl TxPayloadT {
    runtime::tx().guild_identity().deregister()
}

pub fn authorize(authority: Authority, index: bool) -> impl TxPayloadT {
    runtime::tx().guild_identity().authorize(authority, index)
}

pub fn link_address(
    primary: AccountId,
    prefix: Prefix,
    signature: gn_sig::EcdsaSignature,
) -> impl TxPayloadT {
    runtime::tx()
        .guild_identity()
        .link_address(primary, prefix, signature)
}
pub fn unlink_address(prefix: Prefix, address: AccountId) -> impl TxPayloadT {
    runtime::tx()
        .guild_identity()
        .unlink_address(prefix, address)
}

pub fn remove_addresses(prefix: Prefix) -> impl TxPayloadT {
    runtime::tx().guild_identity().remove_addresses(prefix)
}

pub fn link_identity(prefix: Prefix, identity: Identity) -> impl TxPayloadT {
    runtime::tx()
        .guild_identity()
        .link_identity(prefix, identity)
}

pub fn unlink_identity(prefix: Prefix) -> impl TxPayloadT {
    runtime::tx().guild_identity().unlink_identity(prefix)
}

pub fn callback(request_id: u64, result: bool) -> impl TxPayloadT {
    runtime::tx().guild_identity().callback(request_id, result)
}
