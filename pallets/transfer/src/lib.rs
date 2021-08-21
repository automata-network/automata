#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use blake2::digest::{Update, VariableOutput};
    use blake2::VarBlake2b;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, Vec},
        unsigned::ValidateUnsigned,
    };
    use frame_system::{
        offchain::{SendTransactionTypes, SubmitTransaction},
        pallet_prelude::*,
    };
    use primitives::*;
    use sp_core::{ecdsa, H160, U256};
    use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
    use sp_runtime::{traits::UniqueSaturatedInto, SaturatedConversion};
    use sp_std::str;

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
            match call {
                Call::transfer_from_evm_account(ref message, ref signature_raw_bytes) => {
                    if message.len() != 72 {
                        return InvalidTransaction::Custom(1u8).into();
                    }
                    if signature_raw_bytes.len() != 65 {
                        return InvalidTransaction::Custom(2u8).into();
                    }

                    let signature = ecdsa::Signature::from_slice(signature_raw_bytes);

                    let source_address = extract_source_address(message);
                    let source_account_id = Self::evm_address_to_account_id(source_address);

                    // let target_account_id_bytes = extract_target_account_id(message);
                    // let target_account_id = T::AccountId::decode(&mut &target_account_id_bytes[..]).unwrap_or_default();

                    let value = extract_transfer_amount(message);

                    let nonce_bytes = extract_nonce(message);
                    let nonce: T::Index = u32::from_be_bytes(nonce_bytes).into();

                    let result = eth_recover(&signature, message, &[][..])
                        .ok_or(Error::<T>::SignatureInvalid);
                    let address = match result {
                        Ok(address) => address,
                        Err(_e) => {
                            return InvalidTransaction::BadProof.into();
                        }
                    };

                    if address != source_address {
                        return InvalidTransaction::BadProof.into();
                    }

                    let real_nonce: T::Index =
                        frame_system::Module::<T>::account_nonce(&source_account_id);
                    if nonce != real_nonce {
                        return InvalidTransaction::Custom(3u8).into();
                    }

                    let balance = T::Currency::free_balance(&source_account_id);
                    let balance_u256 =
                        U256::from(UniqueSaturatedInto::<u128>::unique_saturated_into(balance));
                    if balance_u256 < U256::from(value) {
                        return InvalidTransaction::Custom(4u8).into();
                    }

                    //do not need `and_requires` because we have already checked nonce
                    ValidTransaction::with_tag_prefix("Automata/evm/substrate/transfer")
                        .priority(UNSIGNED_TXS_PRIORITY)
                        .and_provides((source_address, nonce))
                        .longevity(3)
                        .propagate(true)
                        .build()
                }
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
            message: [u8; 72],
            signature_raw_bytes: [u8; 65],
        ) -> DispatchResultWithPostInfo {
            let signature = ecdsa::Signature::from_slice(&signature_raw_bytes);

            let source_address = extract_source_address(&message);
            let source_account_id = Self::evm_address_to_account_id(source_address);

            let target_account_id_bytes = extract_target_account_id(&message);
            let target_account_id =
                T::AccountId::decode(&mut &target_account_id_bytes[..]).unwrap_or_default();

            let value = extract_transfer_amount(&message);

            let nonce_bytes = extract_nonce(&message);
            let nonce: T::Index = u32::from_be_bytes(nonce_bytes).into();

            let address =
                eth_recover(&signature, &message, &[][..]).ok_or(Error::<T>::SignatureInvalid)?;

            ensure!(address == source_address, Error::<T>::SignatureMismatch);

            let real_nonce: T::Index = frame_system::Module::<T>::account_nonce(&source_account_id);
            ensure!(nonce == real_nonce, Error::<T>::IncorrectNonce);

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

            frame_system::Module::<T>::inc_account_nonce(&source_account_id);
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn submit_unsigned_transaction(
            message: [u8; 72],
            signature_raw_bytes: [u8; 65],
        ) -> Result<(), ()> {
            let call = Call::transfer_from_evm_account(message, signature_raw_bytes);
            SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
        }

        pub fn evm_address_to_account_id(evm_address: H160) -> T::AccountId {
            let hash_bytes = evm_address_to_account_id_bytes(evm_address);
            T::AccountId::decode(&mut &hash_bytes[..]).unwrap_or_default()
        }
    }

    pub fn extract_source_address(message: &[u8; 72]) -> H160 {
        let mut source_address_bytes = [0u8; 20];
        source_address_bytes.copy_from_slice(&message[0..20]);
        source_address_bytes.into()
    }

    pub fn extract_target_account_id(message: &[u8; 72]) -> [u8; 32] {
        let mut target_account_id_bytes = [0u8; 32];
        target_account_id_bytes.copy_from_slice(&message[20..52]);
        target_account_id_bytes
    }

    pub fn extract_transfer_amount(message: &[u8; 72]) -> Balance {
        let mut value_bytes = [0u8; 16];
        value_bytes.copy_from_slice(&message[52..68]);
        let value_128: u128 = u128::from_be_bytes(value_bytes);
        Balance::from(value_128)
    }

    pub fn extract_nonce(message: &[u8; 72]) -> [u8; 4] {
        let mut nonce_bytes = [0u8; 4];
        nonce_bytes.copy_from_slice(&message[68..72]);
        nonce_bytes
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
