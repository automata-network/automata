#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, ReservableCurrency};

pub trait AttestorAccounting {
	type AccountId;
	type Currency: ReservableCurrency<Self::AccountId>;
	fn attestor_staking(&self) -> Result<u32, u32>;
}
