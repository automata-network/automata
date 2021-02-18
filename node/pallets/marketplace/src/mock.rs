// SPDX-License-Identifier: Apache-2.0

use crate::{Config, Module};
use frame_support::{
    impl_outer_dispatch, impl_outer_event, impl_outer_origin, parameter_types, weights::Weight,
};
use frame_system as system;
use pallet_balances as balances;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, Perbill};
//use crate as marketplace;
use crate::Event;
use pallet_stake as stake;
use sp_std::prelude::*;

mod marketplace {
    pub use crate::Event;
}
use fulfillment::Event as geodeEvent;

pub type AccountId = u64;

impl_outer_origin! {
    pub enum Origin for Test {}
}

impl_outer_event! {
    pub enum TestEvent for Test {
        system<T>,
        balances<T>,
        marketplace<T>,
        stake<T>,
        fulfillment<T>,
    }
}

impl_outer_dispatch! {
    pub enum Call for Test where origin: Origin {
        marketplace::Marketplace,
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Call = Call;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = TestEvent;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = ();
    type AccountData = balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type BlockWeights = ();
    type BlockLength = ();
    type SS58Prefix = SS58Prefix;
}

parameter_types! {
    pub const MaxLocks: u32 = 10;
}

impl balances::Config for Test {
    type Balance = u64;
    type DustRemoval = ();
    type Event = TestEvent;
    type ExistentialDeposit = ();
    type AccountStore = System;
    type MaxLocks = MaxLocks;
    type WeightInfo = ();
}
impl fulfillment::Config for Test {
    type Event = TestEvent;
    type GeodeOffReport = fulfillment::GeodeReportOffenceHandler<AccountId, Stake>;
}

impl Config for Test {
    type Event = TestEvent;
    type Commodity = Geode;
}

impl pallet_stake::Config for Test {
    type Event = TestEvent;
    type Currency = Balances;
    type Slash = (); // send the slashed funds to the treasury.
    type Reward = (); // rewards are minted from the void
}

pub type System = frame_system::Module<Test>;
pub type Balances = balances::Module<Test>;
pub type Marketplace = Module<Test>;
pub type Geode = fulfillment::Module<Test>;
pub type Stake = pallet_stake::Module<Test>;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub(crate) fn marketpalce_events() -> Vec<Event<Test>> {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let TestEvent::marketplace(inner) = e {
                Some(inner)
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn geode_events() -> Vec<geodeEvent<Test>> {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let TestEvent::fulfillment(inner) = e {
                Some(inner)
            } else {
                None
            }
        })
        .collect()
}
