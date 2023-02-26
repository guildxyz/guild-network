use super::*;
use gn_common::identity::{eth_hash_message, recover_prehashed, EcdsaSignature};
use sp_core::Pair as PairT;

pub const METADATA: &[u8] =
    &[12u8; <TestRuntime as pallet_guild::Config>::MaxSerializedLen::get() as usize];

pub fn last_event() -> pallet_guild::Event<TestRuntime> {
    System::events()
        .into_iter()
        .filter_map(|e| {
            if let RuntimeEvent::Guild(inner) = e.event {
                Some(inner)
            } else {
                None
            }
        })
        .last()
        .unwrap()
}

pub fn error_msg<'a>(error: DispatchError) -> &'a str {
    match error {
        DispatchError::Module(module_error) => module_error.message.unwrap(),
        DispatchError::BadOrigin => "BadOrigin",
        _ => panic!("unexpected error"),
    }
}

pub fn dummy_answer(
    result: Vec<u8>,
    requester: AccountId,
    request_data: RequestData<AccountId>,
) -> pallet_oracle::OracleAnswer {
    let data = gn_common::Request::<AccountId> {
        requester,
        data: request_data,
    }
    .encode();
    pallet_oracle::OracleAnswer { data, result }
}

pub fn dummy_guild(signer: AccountId, guild_name: GuildName) {
    <Guild>::create_guild(RuntimeOrigin::signed(signer), guild_name, METADATA.to_vec()).unwrap();
    assert_eq!(last_event(), GuildEvent::GuildCreated(signer, guild_name));
    let guild_id = <Guild>::guild_id(guild_name).unwrap();
    let guild = <Guild>::guild(guild_id).unwrap();
    assert_eq!(guild.name, guild_name);
    assert_eq!(guild.owner, signer);
    assert_eq!(guild.metadata, METADATA);
    assert!(guild.roles.is_empty());
}

pub fn dummy_ecdsa_id_with_auth(user: AccountId, seed: [u8; 32]) -> (Identity, EcdsaSignature) {
    let keypair_ecdsa = sp_core::ecdsa::Pair::from_seed_slice(&seed).unwrap();
    let msg = gn_common::utils::verification_msg(user);
    let ecdsa_sig = EcdsaSignature(keypair_ecdsa.sign(msg.as_ref()).0);
    let ecdsa_pubkey = recover_prehashed(eth_hash_message(&msg), &ecdsa_sig).unwrap();
    let ecdsa_address: [u8; 20] = sp_core::keccak_256(&ecdsa_pubkey.serialize_uncompressed()[1..])
        [12..]
        .try_into()
        .unwrap();
    (Identity::Address20(ecdsa_address), ecdsa_sig)
}
