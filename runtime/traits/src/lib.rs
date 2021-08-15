#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, ReservableCurrency,};
use frame_support::dispatch::DispatchResultWithPostInfo;

pub trait AttestorAccounting {
	type AccountId;
	type Currency: ReservableCurrency<Self::AccountId>;
	// type BalanceOf;

	fn attestor_staking(&self, who: Self::AccountId) -> DispatchResultWithPostInfo;
}
