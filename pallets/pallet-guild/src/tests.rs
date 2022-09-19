#![cfg(test)]

use super::*;
use crate as pallet_guild;

use frame_support::{
    assert_noop, assert_ok,
    traits::{ConstU32, ConstU64},
};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Chainlink: pallet_chainlink::{Pallet, Call, Storage, Event<T>},
        Example: example_caller::{Pallet, Call, Storage, Event<T>},
        Guild: pallet_guild::{Pallet, Call, Storage, Event<T>},
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const ExistentialDeposit: u64 = 1;
    pub const MinimumFee: u32 = 500;
    pub const SS58Prefix: u8 = 42;
    pub const ValidityPeriod: u64 = 10;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

#[test]
fn guild_interactions_work() {
    let mut ext: sp_io::TestExternalities = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into();
    ext.execute_with(|| {
        assert_ok!(Guild::create_guild(Origin::signed(4), 444));
        assert!(Guild::guilds(444).is_some());
        assert_ok!(Guild::join_guild(Origin::signed(4), 444));
        assert_eq!(Guild::guilds(444).unwrap().members().len(), 1);
        assert_eq!(Guild::guilds(444).unwrap().members()[0], 4);
        assert_noop!(
            Guild::create_guild(Origin::signed(4), 444),
            Error::<Test>::GuildAlreadyExists
        );
        assert_noop!(
            Guild::create_guild(Origin::signed(5), 444),
            Error::<Test>::GuildAlreadyExists
        );
        assert_noop!(
            Guild::join_guild(Origin::signed(4), 444),
            Error::<Test>::SignerAlreadyJoined
        );
        assert_ok!(Guild::join_guild(Origin::signed(5), 444));
        assert_ok!(Guild::join_guild(Origin::signed(6), 444));
        assert_ok!(Guild::join_guild(Origin::signed(7), 444));
        assert_ok!(Guild::join_guild(Origin::signed(8), 444));
        assert_eq!(Guild::guilds(444).unwrap().members().len(), 5);
        assert_noop!(
            Guild::join_guild(Origin::signed(7), 444),
            Error::<Test>::SignerAlreadyJoined
        );
        assert_noop!(
            Guild::join_guild(Origin::signed(8), 446),
            Error::<Test>::GuildDoesNotExist
        );
        assert_ok!(Guild::create_guild(Origin::signed(1), 446));
        assert_ok!(Guild::join_guild(Origin::signed(8), 446));
    });
}
