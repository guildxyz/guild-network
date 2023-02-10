use super::*;
use gn_common::identity::{eth_hash_message, eth_recover_prehashed};
use sp_core::Pair as PairT;

#[test]
fn unsuccessful_registrations() {
    new_test_ext().execute_with(|| {
        init_chain();
        let user = 0;

        let test_data = vec![
            (
                <Guild>::register(
                    RuntimeOrigin::signed(user),
                    RequestData::ReqCheck {
                        account: 1,
                        guild: [0; 32],
                        role: [1; 32],
                    },
                ),
                "InvalidRequestData",
            ),
            (
                <Guild>::register(
                    RuntimeOrigin::signed(user),
                    RequestData::Register {
                        identity_with_auth: IdentityWithAuth::Other(
                            Identity::Other([0u8; 64]),
                            [0u8; 64],
                        ),
                        index: <TestRuntime as pallet_guild::Config>::MaxIdentities::get(),
                    },
                ),
                "MaxIdentitiesExceeded",
            ),
            (
                <Guild>::register(
                    RuntimeOrigin::signed(user),
                    RequestData::Register {
                        identity_with_auth: IdentityWithAuth::Other(
                            Identity::Other([0u8; 64]),
                            [0u8; 64],
                        ),
                        index: <TestRuntime as pallet_guild::Config>::MaxIdentities::get() - 1,
                    },
                ),
                "NoRegisteredOperators",
            ),
            (
                <Guild>::register(
                    RuntimeOrigin::signed(user),
                    RequestData::Register {
                        identity_with_auth: IdentityWithAuth::Ecdsa(
                            Identity::Address20([0u8; 20]),
                            sp_core::ecdsa::Signature([0u8; 65]),
                        ),
                        index: <TestRuntime as pallet_guild::Config>::MaxIdentities::get() - 1,
                    },
                ),
                "AccessDenied",
            ),
            (
                <Guild>::register(
                    RuntimeOrigin::signed(user),
                    RequestData::Register {
                        identity_with_auth: IdentityWithAuth::Ecdsa(
                            Identity::Address32([0u8; 32]),
                            sp_core::ecdsa::Signature([0u8; 65]),
                        ),
                        index: 0,
                    },
                ),
                "AccessDenied",
            ),
            (
                <Guild>::register(
                    RuntimeOrigin::signed(user),
                    RequestData::Register {
                        identity_with_auth: IdentityWithAuth::Ed25519(
                            Identity::Address32([1u8; 32]),
                            sp_core::ed25519::Signature([0u8; 64]),
                        ),
                        index: 1,
                    },
                ),
                "AccessDenied",
            ),
            (
                <Guild>::register(
                    RuntimeOrigin::signed(user),
                    RequestData::Register {
                        identity_with_auth: IdentityWithAuth::Sr25519(
                            Identity::Address32([1u8; 32]),
                            sp_core::sr25519::Signature([0u8; 64]),
                        ),
                        index: <TestRuntime as pallet_guild::Config>::MaxIdentities::get() - 1,
                    },
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
        init_chain();
        let user = 1;
        let mut index = 0;

        // generate signing keypairs
        let seed = [2u8; 32];
        let keypair_ecdsa = sp_core::ecdsa::Pair::from_seed_slice(&seed).unwrap();
        let keypair_edwards = sp_core::ed25519::Pair::from_seed_slice(&seed).unwrap();
        let keypair_ristretto = sp_core::sr25519::Pair::from_seed_slice(&seed).unwrap();

        // sign message
        let msg = gn_common::utils::verification_msg(user);
        let sig_ecdsa = keypair_ecdsa.sign(msg.as_ref());
        let sig_edwards = keypair_edwards.sign(msg.as_ref());
        let sig_ristretto = keypair_ristretto.sign(msg.as_ref());

        // generate identities with auth
        let ecdsa_pubkey = eth_recover_prehashed(eth_hash_message(&msg), &sig_ecdsa).unwrap();
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
        <Guild>::register(
            RuntimeOrigin::signed(user),
            RequestData::Register {
                identity_with_auth: id_with_auth_ecdsa,
                index,
            },
        )
        .unwrap();
        assert_eq!(last_event(), GuildEvent::IdRegistered(user, index));
        assert_eq!(
            <Guild>::user_data(user).unwrap().get(&index),
            Some(&Identity::Address20(ecdsa_address))
        );
        index += 1;
        <Guild>::register(
            RuntimeOrigin::signed(user),
            RequestData::Register {
                identity_with_auth: id_with_auth_edwards,
                index,
            },
        )
        .unwrap();
        assert_eq!(last_event(), GuildEvent::IdRegistered(user, index));
        assert_eq!(
            <Guild>::user_data(user).unwrap().get(&index),
            Some(&Identity::Address32(keypair_edwards.public().0))
        );
        index += 1;
        <Guild>::register(
            RuntimeOrigin::signed(user),
            RequestData::Register {
                identity_with_auth: id_with_auth_ristretto,
                index,
            },
        )
        .unwrap();
        assert_eq!(
            <Guild>::user_data(user).unwrap().get(&index),
            Some(&Identity::Address32(keypair_ristretto.public().0))
        );
        assert_eq!(last_event(), GuildEvent::IdRegistered(user, index));
        index += 1;
        assert_eq!(<Guild>::user_data(user).unwrap().len(), usize::from(index));
    });
}

#[test]
fn successful_off_chain_registrations() {
    new_test_ext().execute_with(|| {
        init_chain();
        let operator = 0;
        let user = 1;
        let id_zero = Identity::Other([0u8; 64]);
        let id_one = Identity::Other([1u8; 64]);
        let auth = [0u8; 64];
        let index = 0;

        // register an operator first
        <Oracle>::register_operator(RuntimeOrigin::signed(operator)).unwrap();
        // user registers id that requires off-chain verification
        let request_data = RequestData::Register {
            identity_with_auth: IdentityWithAuth::Other(id_zero, auth),
            index,
        };
        <Guild>::register(RuntimeOrigin::signed(user), request_data.clone()).unwrap();
        // pallet receives a dummy oracle answer
        let answer = dummy_answer(vec![u8::from(true)], user, request_data);
        <Guild>::callback(RuntimeOrigin::root(), answer.encode()).unwrap();
        assert_eq!(
            <Guild>::user_data(user).unwrap().get(&index),
            Some(&id_zero)
        );
        assert_eq!(last_event(), GuildEvent::IdRegistered(user, index));
        // user overrides previous id that requires off-chain verification
        let request_data = RequestData::Register {
            identity_with_auth: IdentityWithAuth::Other(id_one, auth),
            index,
        };
        <Guild>::register(RuntimeOrigin::signed(user), request_data.clone()).unwrap();
        // pallet receives a dummy oracle answer
        let answer = dummy_answer(vec![u8::from(true)], user, request_data);
        <Guild>::callback(RuntimeOrigin::root(), answer.encode()).unwrap();
        assert_eq!(<Guild>::user_data(user).unwrap().get(&index), Some(&id_one));
        assert_eq!(<Guild>::user_data(user).unwrap().len(), 1);
        assert_eq!(last_event(), GuildEvent::IdRegistered(user, index));
        // user tries to override again
        let request_data = RequestData::Register {
            identity_with_auth: IdentityWithAuth::Other(id_zero, auth),
            index,
        };
        <Guild>::register(RuntimeOrigin::signed(user), request_data.clone()).unwrap();
        // pallet receives a dummy oracle answer
        let answer = dummy_answer(vec![u8::from(false)], user, request_data);
        assert_eq!(
            error_msg(<Guild>::callback(RuntimeOrigin::root(), answer.encode()).unwrap_err()),
            "AccessDenied"
        );
        assert_eq!(<Guild>::user_data(user).unwrap().get(&index), Some(&id_one));
        assert_eq!(<Guild>::user_data(user).unwrap().len(), 1);
    });
}

#[test]
fn successful_idenity_overrides() {
    new_test_ext().execute_with(|| {
        init_chain();
        let operator = 0;
        let user = 2;
        let seed = [12u8; 32];
        let msg = gn_common::utils::verification_msg(user);
        let keypair_edwards = sp_core::ed25519::Pair::from_seed_slice(&seed).unwrap();
        let sig_edwards = keypair_edwards.sign(msg.as_ref());
        let id_edwards = Identity::Address32(keypair_edwards.public().0);
        let id_zero = Identity::Other([0u8; 64]);
        let id_one = Identity::Other([1u8; 64]);
        let auth = [0u8; 64];
        let index = 1;

        // register an operator first
        <Oracle>::register_operator(RuntimeOrigin::signed(operator)).unwrap();

        // user registers an off-chain-verified identity
        let request_data: RequestData<AccountId> = RequestData::Register {
            identity_with_auth: IdentityWithAuth::Other(id_zero, auth),
            index,
        };
        <Guild>::register(RuntimeOrigin::signed(user), request_data.clone()).unwrap();
        assert!(<Guild>::user_data(user).is_none()); // no id registered yet
        let answer = dummy_answer(vec![u8::from(true)], user, request_data);
        <Guild>::callback(RuntimeOrigin::root(), answer.encode()).unwrap();
        assert_eq!(
            <Guild>::user_data(user).unwrap().get(&index),
            Some(&id_zero)
        );
        assert_eq!(<Guild>::user_data(user).unwrap().len(), 1);
        // user overrides an off-chain-verified identity with an on-chain id
        let request_data: RequestData<AccountId> = RequestData::Register {
            identity_with_auth: IdentityWithAuth::Ed25519(id_edwards, sig_edwards),
            index,
        };
        <Guild>::register(RuntimeOrigin::signed(user), request_data).unwrap();
        assert_eq!(
            <Guild>::user_data(user).unwrap().get(&index),
            Some(&id_edwards)
        );
        assert_eq!(<Guild>::user_data(user).unwrap().len(), 1);
        // user overrides an on-chain-verified identity with an off-chain id
        let request_data: RequestData<AccountId> = RequestData::Register {
            identity_with_auth: IdentityWithAuth::Other(id_one, auth),
            index,
        };
        <Guild>::register(RuntimeOrigin::signed(user), request_data.clone()).unwrap();
        assert_eq!(
            <Guild>::user_data(user).unwrap().get(&index),
            Some(&id_edwards)
        );
        let answer = dummy_answer(vec![u8::from(true)], user, request_data);
        <Guild>::callback(RuntimeOrigin::root(), answer.encode()).unwrap();
        assert_eq!(<Guild>::user_data(user).unwrap().get(&index), Some(&id_one));
        assert_eq!(<Guild>::user_data(user).unwrap().len(), 1);
    });
}
