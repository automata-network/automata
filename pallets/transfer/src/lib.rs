#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::{Currency, ExistenceRequirement, Vec}, unsigned::ValidateUnsigned};
    use frame_system::{pallet_prelude::*, offchain::{SendTransactionTypes, SubmitTransaction}};
    use sp_core::{ecdsa, H160};
    use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
    use sp_runtime::{traits::UniqueSaturatedInto, SaturatedConversion, print};
    use sp_std::str;
    use blake2::digest::{Update, VariableOutput};
    use blake2::VarBlake2b;
    use primitives::*;


    /// Type alias for currency balance.
    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type EcdsaSignature = ecdsa::Signature;

    const UNSIGNED_TXS_PRIORITY: u64 = 100;

    #[pallet::config]
    pub trait Config: SendTransactionTypes<Call<Self>> + frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: Currency<Self::AccountId>;
        type Call: From<Call<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        TransferToEVM(T::AccountId, H160, Balance),
        TransferToSubstrate(H160, T::AccountId, Balance),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Use an invalid attestor id.
        InsufficientBalance,
        SignatureInvalid,
        SignatureMismatch,
        IncorrectNonce,
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            let valid_tx = |provide| ValidTransaction::with_tag_prefix("Automata/evm/transfer")
                .priority(UNSIGNED_TXS_PRIORITY)
                .and_provides([&provide])
                .longevity(3)
                .propagate(true)
                .build();
            
            match call {
                Call::transfer_from_evm_account(_message, _signature_raw_bytes) => valid_tx(b"submit_number_unsigned".to_vec()),
                _ => InvalidTransaction::Call.into(),
            }
        }

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
                ExistenceRequirement::AllowDeath,
            )?;
            Self::deposit_event(Event::TransferToEVM(
                source_account_id,
                target_address,
                value.saturated_into(),
            ));
            Ok(().into())
        }      
        
        #[pallet::weight(0)]
        pub fn transfer_from_evm_account(
            _origin: OriginFor<T>,
            message: [u8; 68],
            signature_raw_bytes: [u8; 65]
        ) -> DispatchResultWithPostInfo {
            let mut signature_bytes = [0u8; 65];
            signature_bytes.copy_from_slice(&signature_raw_bytes);
            let signature = ecdsa::Signature::from_slice(&signature_bytes);
            
            let mut source_address_bytes = [0u8; 20];
            source_address_bytes.copy_from_slice(&message[0..20]);
            let source_address = source_address_bytes.into();
            let source_account_id = Self::evm_address_to_account_id(source_address);
            
            let mut target_account_id_bytes = [0u8; 32];
            target_account_id_bytes.copy_from_slice(&message[20..52]);
            let target_account_id = T::AccountId::decode(&mut &target_account_id_bytes[..]).unwrap_or_default();
            let mut value_bytes = [0u8; 16];
            value_bytes.copy_from_slice(&message[52..68]);
            let value_128: u128 = u128::from_be_bytes(value_bytes);
            let value = Balance::from(value_128);

            let address = eth_recover(&signature, &message, &[][..])
                .ok_or(Error::<T>::SignatureInvalid)?;

            ensure!(
                address == source_address,
                Error::<T>::SignatureMismatch
            );

            T::Currency::transfer(
                &source_account_id,
                &target_account_id,
                value.unique_saturated_into(),
                ExistenceRequirement::AllowDeath,
            )?;

            Self::deposit_event(Event::TransferToSubstrate(
                source_address,
                target_account_id,
                value.saturated_into(),
            ));
            
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn submit_unsigned_transaction(
            message: [u8; 68],
            signature_raw_bytes: [u8; 65]
        ) -> Result<(), ()> {
            let call = Call::transfer_from_evm_account(message, signature_raw_bytes);
            SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(
                call.into()
            )
        }

        pub fn evm_address_to_account_id(evm_address: H160) -> T::AccountId {
            let hash_bytes = evm_address_to_account_id_bytes(evm_address);
            T::AccountId::decode(&mut &hash_bytes[..]).unwrap_or_default()
        }
    }

    pub fn evm_address_to_account_id_bytes(evm_address: H160) -> [u8; 32] {
        let mut data = [0u8; 24];
        data[0..4].copy_from_slice(b"evm:");
        data[4..24].copy_from_slice(&evm_address[..]);
        let mut hasher = VarBlake2b::new(32).unwrap();
        hasher.update(&data);
        let mut hash_bytes = [0u8; 32];
        hasher.finalize_variable(|res| {
            hash_bytes.copy_from_slice(&res[..32]);
        });
        hash_bytes
    }

    pub fn eth_recover(s: &EcdsaSignature, message: &[u8], extra: &[u8]) -> Option<H160> {
        let msg = keccak_256(&ethereum_signable_message(message, extra));
        let mut res = H160::default();
        res.0
            .copy_from_slice(&keccak_256(&secp256k1_ecdsa_recover(&s.0, &msg).ok()?[..])[12..]);
        Some(res)
    }

    fn ethereum_signable_message(what: &[u8], extra: &[u8]) -> Vec<u8> {
        let mut l = what.len() + extra.len();
        let mut rev = Vec::new();
        while l > 0 {
            rev.push(b'0' + (l % 10) as u8);
            l /= 10;
        }
        let mut v = b"\x19Ethereum Signed Message:\n".to_vec();
        v.extend(rev.into_iter().rev());
        v.extend_from_slice(what);
        v.extend_from_slice(extra);
        v
    }
}
