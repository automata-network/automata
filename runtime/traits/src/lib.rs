#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResultWithPostInfo;

pub trait AttestorAccounting {
	type AccountId;
	fn attestor_staking(who: Self::AccountId) -> DispatchResultWithPostInfo;
}
