#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResultWithPostInfo;

pub trait AttestorAccounting {
	type AccountId;
	fn attestor_staking(who: Self::AccountId) -> DispatchResultWithPostInfo;
	fn attestor_unreserve(who: Self::AccountId) -> DispatchResultWithPostInfo;
}

pub trait GeodeAccounting {
	type AccountId;
	fn geode_staking(who: Self::AccountId) -> DispatchResultWithPostInfo;
	fn geode_unreserve(who: Self::AccountId) -> DispatchResultWithPostInfo;
}
