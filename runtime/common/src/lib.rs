#![cfg_attr(not(feature = "std"), no_std)]

pub mod impls;

use frame_support::{
	traits::{Currency},
};

pub type NegativeImbalance<T> = <pallet_balances::Pallet<T> as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;