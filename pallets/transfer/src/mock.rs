// use crate::Config;
// use std::str::FromStr;
// use frame_support::{parameter_types, weights::Weight};
// use frame_system as system;
// use pallet_balances as balances;
// use sp_core::{crypto::AccountId32, H256, H160};
// use sp_runtime::{testing::Header, traits::IdentityLookup, Perbill};
// use sp_io::hashing::keccak_256;

// use crate as transfer;

// type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
// type Block = frame_system::mocking::MockBlock<Test>;

// frame_support::construct_runtime!(
//     pub enum Test where
//         Block = Block,
//         NodeBlock = Block,
//         UncheckedExtrinsic = UncheckedExtrinsic
//     {
//         System: frame_system::{Module, Call, Config, Storage, Event<T>},
//         Transfer: transfer::{Module, Call, Event<T>},
//         Balances: balances::{Module, Call, Storage, Event<T>}
//     }
// );

// parameter_types! {
//     pub const BlockHashCount: u64 = 250;
//     pub const MaximumBlockWeight: Weight = 1024;
//     pub const MaximumBlockLength: u32 = 2 * 1024;
//     pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
//     pub const SS58Prefix: u8 = 42;
// }

// impl frame_system::Config for Test {
//     type BaseCallFilter = ();
//     type Origin = Origin;
//     type Index = u64;
//     type BlockNumber = u64;
//     type Call = Call;
//     type Hash = H256;
//     type Hashing = ::sp_runtime::traits::BlakeTwo256;
//     type AccountId = AccountId32;
//     type Lookup = IdentityLookup<Self::AccountId>;
//     type Header = Header;
//     type Event = Event;
//     type BlockHashCount = BlockHashCount;
//     type DbWeight = ();
//     type Version = ();
//     type PalletInfo = PalletInfo;
//     type AccountData = balances::AccountData<u64>;
//     type OnNewAccount = ();
//     type OnKilledAccount = ();
//     type SystemWeightInfo = ();
//     type BlockWeights = ();
//     type BlockLength = ();
//     type SS58Prefix = SS58Prefix;
// }

// parameter_types! {
//     pub const MaxLocks: u32 = 10;
// }

// impl balances::Config for Test {
//     type Balance = u64;
//     type DustRemoval = ();
//     type Event = Event;
//     type ExistentialDeposit = ();
//     type AccountStore = System;
//     type MaxLocks = MaxLocks;
//     type WeightInfo = ();
// }

// impl Config for Test {
//     type Event = Event;
//     type Currency = Balances;
// }

// pub fn new_test_ext() -> sp_io::TestExternalities {
//     let mut t = system::GenesisConfig::default()
//         .build_storage::<Test>()
//         .unwrap();
//     pallet_balances::GenesisConfig::<Test> {
//         balances: vec![
//             (H160::from_str("1000000000000000000000000000000000000001").unwrap(), 100), 
//             (H160::from_str("1000000000000000000000000000000000000002").unwrap(), 100)
//         ],
//     }
//     .assimilate_storage(&mut t)
//     .unwrap();
//     let mut ext = sp_io::TestExternalities::new(t);
//     ext.execute_with(|| System::set_block_number(1));
//     ext
// }

// pub fn alice() -> secp256k1::SecretKey {
// 	secp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
// }

// pub fn bob() -> secp256k1::SecretKey {
// 	secp256k1::SecretKey::parse(&keccak_256(b"Bob")).unwrap()
// }

// pub fn bob_account_id() -> AccountId32 {
// 	let address = Transfer::evm_address_to_account_id(&bob());
// 	let mut data = [0u8; 32];
// 	data[0..4].copy_from_slice(b"evm:");
// 	data[4..24].copy_from_slice(&address[..]);
// 	AccountId32::from(Into::<[u8; 32]>::into(data))
// }