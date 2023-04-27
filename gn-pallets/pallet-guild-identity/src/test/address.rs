use super::*;
use gn_sig::webcrypto::hash_pubkey;
use gn_sig::webcrypto::wallet::Wallet;
use parity_scale_codec::Encode;

#[test]
fn link_and_unlink_addresses() {
    new_test_ext().execute_with(|| {
        let primary_account: <TestRuntime as frame_system::Config>::AccountId = 99;
        let linked_account_0: <TestRuntime as frame_system::Config>::AccountId = 100;
        let linked_account_1: <TestRuntime as frame_system::Config>::AccountId = 101;
        let linked_account_2: <TestRuntime as frame_system::Config>::AccountId = 102;
        let linked_account_3: <TestRuntime as frame_system::Config>::AccountId = 103;

        let wallet = Wallet::from_seed([10u8; 32]).unwrap();
        let authority_0 = hash_pubkey(&wallet.pubkey());
        let wallet = Wallet::from_seed([12u8; 32]).unwrap();
        let authority_1 = hash_pubkey(&wallet.pubkey());
        let prefix_0 = [0u8; 8];
        let prefix_1 = [1u8; 8];
        let prefix_2 = [2u8; 8];

        // trying to link address without registering first
        let signature = wallet.sign(linked_account_0.encode()).unwrap();
        assert_noop!(
            <GuildIdentity>::link_address(
                RuntimeOrigin::signed(linked_account_0),
                primary_account,
                prefix_0,
                signature,
            ),
            IdentityError::AccountDoesNotExist
        );

        // register
        assert_ok!(<GuildIdentity>::register(RuntimeOrigin::signed(
            primary_account
        )));
        // authorize
        assert_ok!(<GuildIdentity>::authorize(
            RuntimeOrigin::signed(primary_account),
            authority_0,
            true
        ));
        assert_ok!(<GuildIdentity>::authorize(
            RuntimeOrigin::signed(primary_account),
            authority_1,
            false
        ));

        // link first address
        assert_ok!(<GuildIdentity>::link_address(
            RuntimeOrigin::signed(linked_account_0),
            primary_account,
            prefix_0,
            signature
        ));
        assert_eq!(
            last_event(),
            IdentityEvent::AddressLinked(primary_account, prefix_0, linked_account_0)
        );
        // link second address under the same prefix
        let signature = wallet.sign(linked_account_1.to_le_bytes()).unwrap();
        assert_ok!(<GuildIdentity>::link_address(
            RuntimeOrigin::signed(linked_account_1),
            primary_account,
            prefix_0,
            signature
        ));
        assert_eq!(
            last_event(),
            IdentityEvent::AddressLinked(primary_account, prefix_0, linked_account_1)
        );
        // try to link the same address again under the same prefix
        assert_noop!(
            <GuildIdentity>::link_address(
                RuntimeOrigin::signed(linked_account_1),
                primary_account,
                prefix_0,
                signature
            ),
            IdentityError::AddressAlreadyLinked
        );
        // try to link another address to the same prefix
        let signature = wallet.sign(linked_account_2.to_le_bytes()).unwrap();
        assert_noop!(
            <GuildIdentity>::link_address(
                RuntimeOrigin::signed(linked_account_2),
                primary_account,
                prefix_0,
                signature
            ),
            IdentityError::MaxLinkedAddressesExceeded
        );
        // link two addresses under another prefix
        assert_ok!(<GuildIdentity>::link_address(
            RuntimeOrigin::signed(linked_account_2),
            primary_account,
            prefix_1,
            signature
        ));
        let signature = wallet.sign(linked_account_3.to_le_bytes()).unwrap();
        assert_ok!(<GuildIdentity>::link_address(
            RuntimeOrigin::signed(linked_account_3),
            primary_account,
            prefix_1,
            signature
        ));
        // try to add a new address type
        let signature = wallet.sign(linked_account_0.to_le_bytes()).unwrap();
        assert_noop!(
            <GuildIdentity>::link_address(
                RuntimeOrigin::signed(linked_account_0),
                primary_account,
                prefix_2,
                signature
            ),
            IdentityError::MaxLinkedAddressTypesExceeded
        );
        // check current linked addresses
        let address_map = <GuildIdentity>::addresses(primary_account).unwrap();
        assert_eq!(address_map.len(), 2);
        assert_eq!(
            address_map.get(&prefix_0).unwrap().to_vec(),
            &[linked_account_0, linked_account_1]
        );
        assert_eq!(
            address_map.get(&prefix_1).unwrap().to_vec(),
            &[linked_account_2, linked_account_3]
        );
        // unlink an address from prefix_1 and delete prefix_0 altogether
        assert_ok!(<GuildIdentity>::unlink_address(
            RuntimeOrigin::signed(primary_account),
            prefix_1,
            linked_account_2,
        ));
        assert_eq!(
            last_event(),
            IdentityEvent::AddressUnlinked(primary_account, prefix_1, linked_account_2)
        );
        assert_ok!(<GuildIdentity>::remove_addresses(
            RuntimeOrigin::signed(primary_account),
            prefix_0,
        ));
        assert_eq!(
            last_event(),
            IdentityEvent::AddressesRemoved(primary_account, prefix_0)
        );
        // check current linked addresses
        let address_map = <GuildIdentity>::addresses(primary_account).unwrap();
        assert_eq!(address_map.len(), 1);
        assert!(address_map.get(&prefix_0).is_none());
        assert_eq!(
            address_map.get(&prefix_1).unwrap().to_vec(),
            &[linked_account_3]
        );
        // link new addresses and add a new prefix
        let signature = wallet.sign(linked_account_0.to_le_bytes()).unwrap();
        assert_ok!(<GuildIdentity>::link_address(
            RuntimeOrigin::signed(linked_account_0),
            primary_account,
            prefix_1,
            signature
        ));
        let signature = wallet.sign(linked_account_2.to_le_bytes()).unwrap();
        assert_ok!(<GuildIdentity>::link_address(
            RuntimeOrigin::signed(linked_account_2),
            primary_account,
            prefix_2,
            signature
        ));
        let signature = wallet.sign(linked_account_1.to_le_bytes()).unwrap();
        assert_ok!(<GuildIdentity>::link_address(
            RuntimeOrigin::signed(linked_account_1),
            primary_account,
            prefix_2,
            signature
        ));
        // check current linked addresses
        let address_map = <GuildIdentity>::addresses(primary_account).unwrap();
        assert_eq!(address_map.len(), 2);
        assert_eq!(
            address_map.get(&prefix_2).unwrap().to_vec(),
            &[linked_account_2, linked_account_1]
        );
        assert_eq!(
            address_map.get(&prefix_1).unwrap().to_vec(),
            &[linked_account_3, linked_account_0]
        );
        // invalid unlink/delete transactions
        let failing_transactions = vec![
            (
                <GuildIdentity>::link_address(
                    RuntimeOrigin::signed(linked_account_0),
                    primary_account,
                    prefix_1,
                    [0u8; 65],
                ),
                IdentityError::InvalidAuthoritySignature,
            ),
            (
                <GuildIdentity>::link_address(
                    RuntimeOrigin::signed(linked_account_0),
                    primary_account,
                    prefix_1,
                    signature,
                ),
                IdentityError::UnknownAuthority,
            ),
            (
                <GuildIdentity>::unlink_address(
                    RuntimeOrigin::signed(linked_account_0),
                    prefix_0,
                    linked_account_2,
                ),
                IdentityError::AccountDoesNotExist,
            ),
            (
                <GuildIdentity>::unlink_address(
                    RuntimeOrigin::signed(primary_account),
                    prefix_0,
                    linked_account_2,
                ),
                IdentityError::AddressPrefixDoesNotExist,
            ),
            (
                <GuildIdentity>::unlink_address(
                    RuntimeOrigin::signed(primary_account),
                    prefix_1,
                    linked_account_2,
                ),
                IdentityError::AddressDoesNotExist,
            ),
            (
                <GuildIdentity>::remove_addresses(
                    RuntimeOrigin::signed(linked_account_0),
                    prefix_1,
                ),
                IdentityError::AccountDoesNotExist,
            ),
            (
                <GuildIdentity>::remove_addresses(RuntimeOrigin::signed(primary_account), prefix_0),
                IdentityError::AddressPrefixDoesNotExist,
            ),
        ];

        for (tx, error) in failing_transactions {
            assert_noop!(tx, error);
        }
    });
}
