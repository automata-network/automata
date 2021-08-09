#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

// #[cfg(test)]
// mod tests;



#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use frame_support::traits::{Currency, ExistenceRequirement, Vec};
    use sp_core::{H160, ecdsa, U256};
    // use sp_runtime::AccountId32;
    use sp_io::{
        crypto::secp256k1_ecdsa_recover, 
        hashing::keccak_256
    };
    use blake2::VarBlake2b;
    use blake2::digest::{Update, VariableOutput};
    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};
    // use primitives::AccountId;

    /// Type alias for currency balance.
    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type EcdsaSignature = ecdsa::Signature;

    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Eq, Clone, RuntimeDebug, Default)]
    pub struct TransferParam<AccountId> {
        pub source_address: H160,
        pub target_address: AccountId,
        pub value: u32,
        pub signature: ecdsa::Signature
        //shoule we maintain a nonce for each user?
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: Currency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {
        TransferToEVM(T::AccountId, H160, U256),
        TransferToSubstrate(H160, T::AccountId, U256),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Use an invalid attestor id.
        InsufficientBalance,
        SignatureInvalid,
        SignatureMismatch,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        //transfer from a substrate account to evm account, source address is the address who sign the extrinsics
        #[pallet::weight(0)]
        pub fn transfer_to_evm_account(
            origin: OriginFor<T>,
            target_address: H160,
            value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let source_account_id = ensure_signed(origin)?;
            let target_account_id = Self::evm_address_to_account_id(target_address);
            T::Currency::transfer(
				&source_account_id,
				&target_account_id,
				value,
				ExistenceRequirement::AllowDeath
			)?;
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        //transfer from a evm account to substrate account, target address is the address who sign the extrinsics
        pub fn transfer_from_evm_account(
            param: TransferParam<T::AccountId>
        ) -> DispatchResultWithPostInfo {
            let target_account_id = param.target_address;
            let source_account_id = Self::evm_address_to_account_id(param.source_address);
            
            let address = Self::eth_recover(&param.signature, &target_account_id.using_encoded(Self::to_ascii_hex), &[][..])
                .ok_or(Error::<T>::SignatureInvalid)?;
            ensure!(address == param.source_address, Error::<T>::SignatureMismatch);
            
            T::Currency::transfer(
				&source_account_id,
				&target_account_id,
                param.value.into(),
				ExistenceRequirement::AllowDeath
			)?;
            Ok(().into())
        }

        fn evm_address_to_account_id(evm_address: H160) -> T::AccountId {
            let mut data = [0u8; 24];
            data[0..4].copy_from_slice(b"evm:");
            data[4..24].copy_from_slice(&evm_address[..]);
            let mut hasher = VarBlake2b::new(32).unwrap();
            hasher.update(&data);
            let hash = hasher.finalize_boxed();
            let mut hash_bytes = [0u8, 32];
            hash_bytes.copy_from_slice(&hash);

            T::AccountId::decode(&mut &hash_bytes[..]).unwrap_or_default()
        }

        fn eth_recover(s: &EcdsaSignature, message: &[u8], extra: &[u8]) -> Option<H160> {
            let msg = keccak_256(&Self::ethereum_signable_message(message, extra));
            let mut res = H160::default();
            res.0.copy_from_slice(&keccak_256(&secp256k1_ecdsa_recover(&s.0, &msg).ok()?[..])[12..]);
            Some(res)
        }

        fn ethereum_signable_message(what: &[u8], extra: &[u8]) -> Vec<u8> {
            let prefix = b"automata:transfer_from_evm_account:";
            let mut l = prefix.len() + what.len() + extra.len();
            let mut rev = Vec::new();
            while l > 0 {
                rev.push(b'0' + (l % 10) as u8);
                l /= 10;
            }
            let mut v = b"\x19Ethereum Signed Message:\n".to_vec();
            v.extend(rev.into_iter().rev());
            v.extend_from_slice(&prefix[..]);
            v.extend_from_slice(what);
            v.extend_from_slice(extra);
            v
        }

        fn to_ascii_hex(data: &[u8]) -> Vec<u8> {
            let mut r = Vec::with_capacity(data.len() * 2);
            let mut push_nibble = |n| r.push(if n < 10 { b'0' + n } else { b'a' - 10 + n });
            for &b in data.iter() {
                push_nibble(b / 16);
                push_nibble(b % 16);
            }
            r
        }
    }
}