use super::*;
use frame_support::pallet_prelude::BoundedVec;
use gn_sig::webcrypto::hash_pubkey;
use gn_sig::webcrypto::wallet::Wallet;

#[test]
fn link_and_unlink_addresses() {
    new_test_ext().execute_with(|| {
        let primary_account: <TestRuntime as frame_system::Config>::AccountId = 99;
        let linked_account: <TestRuntime as frame_system::Config>::AccountId = 101;
        let wallet = Wallet::from_seed([12u8; 32]).unwrap();
        let authority_1 = hash_pubkey(&wallet.pubkey());
        let prefix_0 = [0u8; 8];
        let mut expected = BoundedVec::with_max_capacity();

        // register
        assert_ok!(<GuildIdentity>::register(RuntimeOrigin::signed(
            primary_account
        )));
        // authorize
        assert_ok!(<GuildIdentity>::authorize(
            RuntimeOrigin::signed(primary_account),
            authority_1,
            false
        ));

        // link first address
        let signature = wallet.sign(&linked_account.to_le_bytes()).unwrap();
        assert_ok!(<GuildIdentity>::link_address(
            RuntimeOrigin::signed(linked_account),
            primary_account,
            prefix_0,
            signature
        ));
        assert_eq!(
            last_event(),
            IdentityEvent::AddressLinked(primary_account, prefix_0, linked_account)
        );
        expected.try_push(linked_account).unwrap();
        assert_eq!(
            <GuildIdentity>::addresses(primary_account)
                .unwrap()
                .get(&prefix_0),
            Some(&expected)
        );
    });
}
