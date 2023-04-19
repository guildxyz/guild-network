use super::*;
use gn_common::identity::*;
use sp_core::Pair as PairT;

#[test]
fn unsuccessful_registrations() {
    new_test_ext().execute_with(|| {
        let user = 0;
        let max_identities = <TestRuntime as pallet_guild::Config>::MaxIdentities::get();

        let test_data = vec![
            (
                <Guild>::register(
                    RuntimeOrigin::none(),
                    IdentityWithAuth::Other(Identity::Other([0u8; 64]), [0u8; 64]),
                    0,
                ),
                "BadOrigin",
            ),
            (
                <Guild>::register(
                    RuntimeOrigin::root(),
                    IdentityWithAuth::Other(Identity::Other([0u8; 64]), [0u8; 64]),
                    0,
                ),
                "BadOrigin",
            ),
            (
                <Guild>::register(
                    RuntimeOrigin::signed(user),
                    IdentityWithAuth::Other(Identity::Other([0u8; 64]), [0u8; 64]),
                    max_identities,
                ),
                "MaxIdentitiesExceeded",
            ),
            (
                <Guild>::register(
                    RuntimeOrigin::signed(user),
                    IdentityWithAuth::Other(Identity::Other([0u8; 64]), [0u8; 64]),
                    max_identities - 1,
                ),
                "NoActiveOperators",
            ),
            (
                <Guild>::register(
                    RuntimeOrigin::signed(user),
                    IdentityWithAuth::Ecdsa(
                        Identity::Address20([0u8; 20]),
                        EcdsaSignature([0u8; 65]),
                    ),
                    max_identities - 1,
                ),
                "AccessDenied",
            ),
            (
                <Guild>::register(
                    RuntimeOrigin::signed(user),
                    IdentityWithAuth::Ecdsa(
                        Identity::Address32([0u8; 32]),
                        EcdsaSignature([0u8; 65]),
                    ),
                    0,
                ),
                "AccessDenied",
            ),
            (
                <Guild>::register(
                    RuntimeOrigin::signed(user),
                    IdentityWithAuth::Ed25519(
                        Identity::Address32([1u8; 32]),
                        Ed25519Signature([0u8; 64]),
                    ),
                    1,
                ),
                "AccessDenied",
            ),
            (
                <Guild>::register(
                    RuntimeOrigin::signed(user),
                    IdentityWithAuth::Sr25519(
                        Identity::Address32([1u8; 32]),
                        Sr25519Signature([0u8; 64]),
                    ),
                    max_identities - 1,
                ),
                "AccessDenied",
            ),
        ];

        for (extrinsic, raw_error) in test_data {
            assert_eq!(error_msg(extrinsic.unwrap_err()), raw_error);
        }
    });
}

#[test]
fn successful_on_chain_registrations() {
    new_test_ext().execute_with(|| {
        let user = 1;
        let mut index = 0;

        // generate signing keypairs
        let seed = [2u8; 32];
        let keypair_ecdsa = sp_core::ecdsa::Pair::from_seed_slice(&seed).unwrap();
        let keypair_edwards = sp_core::ed25519::Pair::from_seed_slice(&seed).unwrap();
        let keypair_ristretto = sp_core::sr25519::Pair::from_seed_slice(&seed).unwrap();

        // sign message
        let msg = gn_common::utils::verification_msg(user);
        let sig_ecdsa = EcdsaSignature(keypair_ecdsa.sign(msg.as_ref()).0);
        let sig_edwards = Ed25519Signature(keypair_edwards.sign(msg.as_ref()).0);
        let sig_ristretto = Sr25519Signature(keypair_ristretto.sign(msg.as_ref()).0);

        // generate identities with auth
        let ecdsa_pubkey = recover_prehashed(eth_hash_message(&msg), &sig_ecdsa.0).unwrap();
        let ecdsa_address: [u8; 20] =
            sp_core::keccak_256(&ecdsa_pubkey.serialize_uncompressed()[1..])[12..]
                .try_into()
                .unwrap();

        let id_with_auth_ecdsa =
            IdentityWithAuth::Ecdsa(Identity::Address20(ecdsa_address), sig_ecdsa);
        let id_with_auth_edwards =
            IdentityWithAuth::Ed25519(Identity::Address32(keypair_edwards.public().0), sig_edwards);
        let id_with_auth_ristretto = IdentityWithAuth::Sr25519(
            Identity::Address32(keypair_ristretto.public().0),
            sig_ristretto,
        );

        // register various identities for user
        <Guild>::register(RuntimeOrigin::signed(user), id_with_auth_ecdsa, index).unwrap();
        assert_eq!(last_event(), GuildEvent::IdRegistered(user, index));
        assert_eq!(
            <Guild>::user_data(user, index),
            Some(Identity::Address20(ecdsa_address))
        );
        index += 1;
        <Guild>::register(RuntimeOrigin::signed(user), id_with_auth_edwards, index).unwrap();
        assert_eq!(last_event(), GuildEvent::IdRegistered(user, index));
        assert_eq!(
            <Guild>::user_data(user, index),
            Some(Identity::Address32(keypair_edwards.public().0))
        );
        index += 1;
        <Guild>::register(RuntimeOrigin::signed(user), id_with_auth_ristretto, index).unwrap();
        assert_eq!(
            <Guild>::user_data(user, index),
            Some(Identity::Address32(keypair_ristretto.public().0))
        );
        assert_eq!(last_event(), GuildEvent::IdRegistered(user, index));
    });
}

#[test]
fn successful_off_chain_registrations() {
    new_test_ext().execute_with(|| {
        let operator = 0;
        let user = 1;
        let id_zero = Identity::Other([0u8; 64]);
        let id_one = Identity::Other([1u8; 64]);
        let auth = [0u8; 64];
        let index = 0;
        let id_auth_zero = IdentityWithAuth::Other(id_zero, auth);
        let id_auth_one = IdentityWithAuth::Other(id_one, auth);

        // register an operator first
        <Oracle>::register_operator(RuntimeOrigin::root(), operator).unwrap();
        <Oracle>::activate_operator(RuntimeOrigin::signed(operator)).unwrap();
        // user registers id that requires off-chain verification
        <Guild>::register(RuntimeOrigin::signed(user), id_auth_zero, index).unwrap();
        // pallet receives a dummy oracle answer
        let request_data = RequestData::Register {
            identity_with_auth: id_auth_zero,
            index,
        };
        let answer = dummy_answer(vec![u8::from(true)], user, request_data);
        <Guild>::callback(RuntimeOrigin::root(), answer.encode()).unwrap();
        assert_eq!(<Guild>::user_data(user, index), Some(id_zero));
        assert_eq!(last_event(), GuildEvent::IdRegistered(user, index));
        // user overrides previous id that requires off-chain verification
        <Guild>::register(RuntimeOrigin::signed(user), id_auth_one, index).unwrap();
        // pallet receives a dummy oracle answer
        let request_data = RequestData::Register {
            identity_with_auth: id_auth_one,
            index,
        };
        let answer = dummy_answer(vec![u8::from(true)], user, request_data);
        <Guild>::callback(RuntimeOrigin::root(), answer.encode()).unwrap();
        assert_eq!(<Guild>::user_data(user, index), Some(id_one));
        assert_eq!(last_event(), GuildEvent::IdRegistered(user, index));
        // user tries to override again
        <Guild>::register(RuntimeOrigin::signed(user), id_auth_zero, index).unwrap();
        // pallet receives a dummy oracle answer
        let request_data = RequestData::Register {
            identity_with_auth: id_auth_zero,
            index,
        };
        let answer = dummy_answer(vec![u8::from(false)], user, request_data);
        assert_eq!(
            error_msg(<Guild>::callback(RuntimeOrigin::root(), answer.encode()).unwrap_err()),
            "AccessDenied"
        );
        assert_eq!(<Guild>::user_data(user, index), Some(id_one));
    });
}

#[test]
fn successful_idenity_overrides() {
    new_test_ext().execute_with(|| {
        let operator = 0;
        let user = 2;
        let seed = [12u8; 32];
        let msg = gn_common::utils::verification_msg(user);
        let keypair_edwards = sp_core::ed25519::Pair::from_seed_slice(&seed).unwrap();
        let sig_edwards = Ed25519Signature(keypair_edwards.sign(msg.as_ref()).0);
        let id_edwards = Identity::Address32(keypair_edwards.public().0);
        let id_zero = Identity::Other([0u8; 64]);
        let id_one = Identity::Other([1u8; 64]);
        let auth = [0u8; 64];
        let index = 1;

        // register an operator first
        <Oracle>::register_operator(RuntimeOrigin::root(), operator).unwrap();
        <Oracle>::activate_operator(RuntimeOrigin::signed(operator)).unwrap();
        // user registers an off-chain-verified identity
        let identity_with_auth = IdentityWithAuth::Other(id_zero, auth);
        let request_data: RequestData<AccountId> = RequestData::Register {
            identity_with_auth,
            index,
        };
        <Guild>::register(RuntimeOrigin::signed(user), identity_with_auth, index).unwrap();
        assert!(<Guild>::user_data(user, index).is_none()); // no id registered yet
        let answer = dummy_answer(vec![u8::from(true)], user, request_data);
        <Guild>::callback(RuntimeOrigin::root(), answer.encode()).unwrap();
        assert_eq!(<Guild>::user_data(user, index), Some(id_zero));
        // user overrides an off-chain-verified identity with an on-chain id
        let identity_with_auth = IdentityWithAuth::Ed25519(id_edwards, sig_edwards);
        <Guild>::register(RuntimeOrigin::signed(user), identity_with_auth, index).unwrap();
        assert_eq!(<Guild>::user_data(user, index), Some(id_edwards));
        // user overrides an on-chain-verified identity with an off-chain id
        let identity_with_auth = IdentityWithAuth::Other(id_one, auth);
        let request_data: RequestData<AccountId> = RequestData::Register {
            identity_with_auth,
            index,
        };
        <Guild>::register(RuntimeOrigin::signed(user), identity_with_auth, index).unwrap();
        assert_eq!(<Guild>::user_data(user, index), Some(id_edwards));
        let answer = dummy_answer(vec![u8::from(true)], user, request_data);
        <Guild>::callback(RuntimeOrigin::root(), answer.encode()).unwrap();
        assert_eq!(<Guild>::user_data(user, index), Some(id_one));
    });
}
