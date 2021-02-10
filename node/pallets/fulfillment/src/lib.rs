// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use dispatch::{DispatchError, DispatchResult};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, traits::Currency,
};
use frame_system::{ensure_root, ensure_signed};
use sp_runtime::{RuntimeDebug, SaturatedConversion};
use sp_std::prelude::*;

pub use property::{GeodeProperties, GeodeProperty};
mod slash;
pub use slash::{GeodeOffenceTrait, GeodeReport, GeodeReportOffenceHandler};

#[cfg(test)]
mod mock;
mod property;

#[cfg(test)]
mod tests;

pub const ATTESTOR_REQUIRE: usize = 1;
pub const TIMELIMIT: u64 = 30;

/// Geode state
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum GeodeState {
    /// The init state when provider register the geode.
    Registered,
    /// When geode get enough attestors' attestation, it turns to Attested.
    Attested,
    /// When geode is put into market.
    InOrder,
    /// When the geode is sold out and be working.
    InWork,
}

/// The geode struct shows its status.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Geode<AccountId, Hash> {
    /// Geode id.
    pub owner: AccountId,
    /// When somebody buy the geode, he'll be stored here.
    pub user: Option<AccountId>,
    /// The provider of the geode.
    pub provider: Option<AccountId>,
    /// Geode's ip.
    pub ip: Vec<u8>,
    /// Geode's dns.
    pub dns: Vec<u8>,
    /// Geode's concrete properties.
    pub props: Option<GeodeProperties>,
    /// The binary hash(stored in IPFS). Geode will get the binary and run it.
    pub binary: Option<Hash>,
    /// The attestors for this geode.
    pub attestors: Vec<AccountId>,
    /// Current state of the geode.
    pub state: GeodeState,
}

/// Attestor struct
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Attestor {
    /// Attestor's url, geode will get it and communicate with attestor.
    pub url: Vec<u8>,
    /// Attestor's Secp256r1PublicKey
    pub pubkey: Vec<u8>,
}

/// Register Geode struct, used when provider register geode.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct RegisterGeode<AccountId, Hash> {
    /// The block number when the geode is registered.
    pub start: u64,
    /// Geode record.
    pub geode_record: Geode<AccountId, Hash>,
}

pub type RegisterOf<T> =
    RegisterGeode<<T as frame_system::Config>::AccountId, <T as frame_system::Config>::Hash>;
pub type GeodeOf<T> =
    Geode<<T as frame_system::Config>::AccountId, <T as frame_system::Config>::Hash>;

pub type BalanceOf<T> = <<T as pallet_stake::Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;

pub trait Config: frame_system::Config + pallet_stake::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type GeodeOffReport: GeodeReport<Self>;
}

decl_storage! {
    trait Store for Module<T: Config> as fulfillment {
        /// Registered geodes. They need attestors to be valid geode.
        RegisterGeodes get(fn register_geodes): map hasher(twox_64_concat) T::AccountId => Option<RegisterOf<T>>;
        /// Valid geodes which get enough attestors.
        Geodes get(fn geodes): map hasher(twox_64_concat) T::AccountId => Option<GeodeOf<T>>;
        /// Record of registered attestors.
        Attestors get(fn attestors): map hasher(twox_64_concat) T::AccountId => Option<Attestor>;
        /// Min limit of stake to be an attestor.
        AttStakeMin get(fn att_stake_min): BalanceOf<T>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        /// Somebody report a misconduct. \[reporter, offender\]
        ReportBlame(AccountId, AccountId),
        /// Attestor registered. \[attestor_id\]
        AttestorRegister(AccountId),
        /// Attestor moved. \[attestor_id\]
        AttestorRemove(AccountId),
        /// Provider register geode. \[provider_id, geode_id\]
        GeodeRegister(AccountId, AccountId),
        /// A geode is removed. \[geode_id\]
        GeodeRemove(AccountId),
        /// Geode's record updated. \[geode_id\]
        GeodeUpdate(AccountId),
        /// User set the binary hash to his geode. \[geode_id\]
        SetBinary(AccountId),
        /// Attestor attestor a geode. \[attestor_id, geode_id\]
        AttestFor(AccountId, AccountId),
        /// Geode's dns updated. \[geode_id\]
        DnsUpdate(AccountId),
        /// Attestor's url updated. \[attestor_id\]
        AttestorUpdate(AccountId),
        /// Geodes which didn't get enough attestors at limited time after registered.
        /// \[Vec<geode_id>\]
        AttestTimeOut(Vec<AccountId>),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Duplicate attestor for geode.
        AlreadyAttestFor,
        /// Duplicate register geode.
        AlreadyGeode,
        /// Use an invalid attestor id.
        InvalidAttestor,
        /// Use an invalid geode id.
        InvalidGeode,
        /// The GeodeState can't allow you to do something now.
        InvalidGeodeState,
        /// You doesn't have the right to do what you want.
        NoRight,
        /// The geode is in work so you can't do this.
        GeodeInWork,
        /// The geode is in the market so you can't do this.
        GeodeInOrder,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Show that I'm in work. Not in use now.
        #[weight = 0]
        pub fn im_online(origin, _proof: Vec<u8>) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            // TODO: deal the challenge
            Ok(())
        }

        /// Report that somebody did a misconduct. The actual usage is being considered.
        #[weight = 0]
        pub fn report_misconduct(origin, geode_id: T::AccountId, _proof: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::report_geode_offence(who.clone(), geode_id.clone());
            Self::deposit_event(RawEvent::ReportBlame(who, geode_id));
            Ok(())
        }

        /// Called by provider to remove geode .
        /// Return Ok() only when the geode's state is Registered/Attested
        #[weight = 0]
        pub fn geode_remove(origin, geode: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            if <Geodes<T>>::contains_key(&geode) {
                let geode_use = <Geodes<T>>::get(&geode).ok_or(Error::<T>::InvalidGeode)?;
                ensure!(geode_use.provider == Some(who.clone()), Error::<T>::NoRight);
                ensure!(geode_use.state == GeodeState::Attested, Error::<T>::InvalidGeodeState);
                <Geodes<T>>::remove(&geode);
            } else if <RegisterGeodes<T>>::contains_key(&geode) {
                let register_geode = <RegisterGeodes<T>>::get(&geode).ok_or(Error::<T>::InvalidGeode)?;
                ensure!(register_geode.geode_record.provider == Some(who.clone()), Error::<T>::NoRight);
                ensure!(register_geode.geode_record.state == GeodeState::Registered, Error::<T>::InvalidGeodeState);
                <RegisterGeodes<T>>::remove(&geode);
            } else {
                return Err(DispatchError::from(Error::<T>::InvalidGeode));
            }

            Self::deposit_event(RawEvent::GeodeRemove(geode));
            Ok(())
        }

        /// Called by provider to update geode's record
        ///
        /// #note
        ///
        /// The user/state/attestor should not be managed by provider,
        /// and change the provider is dangerous, once you change the provider,
        /// you won't own the control of the geode.
        #[weight = 0]
        pub fn geode_update(origin, geode: T::AccountId, record: GeodeOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let geode_use = <Geodes<T>>::get(&geode).ok_or(Error::<T>::InvalidGeode)?;
            ensure!(geode_use.provider == Some(who), Error::<T>::NoRight);
            ensure!(geode_use.state != GeodeState::InWork, Error::<T>::GeodeInWork);
            let mut record = record;
            record.attestors = geode_use.attestors;
            record.state = geode_use.state;
            record.user = geode_use.user;
            <Geodes<T>>::insert(geode.clone(), record);
            Self::deposit_event(RawEvent::GeodeUpdate(geode));
            Ok(())
        }

        /// Register as an attestor.
        #[weight = 0]
        pub fn attestor_register(origin, url: Vec<u8>, pubkey: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let limit = <AttStakeMin<T>>::get();
            <pallet_stake::Module<T>>::enough_stake(&who,limit)?;
            let attestor = Attestor {
                url,
                pubkey
            };
             <Attestors<T>>::insert(who.clone(), attestor);
             Self::deposit_event(RawEvent::AttestorRegister(who));
             Ok(())
        }

        /// Remove self from attestors.
        ///
        ///  #note
        ///
        /// Currently, we use reliable attestor which would not do misconducts.
        /// This function should be called when the attestor is not serving for any geode.
        #[weight = 0]
        pub fn attestor_remove(origin) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(<Attestors<T>>::contains_key(&who), Error::<T>::InvalidAttestor);
            <Attestors<T>>::remove(&who);
            Self::deposit_event(RawEvent::AttestorRemove(who));
            Ok(())
        }

        /// Called by attestor to update its url.
        #[weight = 0]
        pub fn attestor_update(origin, data: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut attestor = <Attestors<T>>::get(&who).ok_or(Error::<T>::InvalidAttestor)?;
            attestor.url = data;
            <Attestors<T>>::insert(who.clone(), attestor);
            Self::deposit_event(RawEvent::AttestorUpdate(who));
            Ok(())
        }

        /// Called by attestor to attest Geode.
        #[weight = 0]
        pub fn attestor_attest_geode(origin, geode: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(<Attestors<T>>::contains_key(&who), Error::<T>::InvalidAttestor);

            let mut register_geode = <RegisterGeodes<T>>::get(&geode).ok_or(Error::<T>::InvalidGeode)?;
            for att in register_geode.geode_record.attestors.clone() {
                ensure!(att != who, Error::<T>::AlreadyAttestFor);
            }
            register_geode.geode_record.attestors.push(who.clone());
            <RegisterGeodes<T>>::insert(geode.clone(), register_geode);

            Self::deposit_event(RawEvent::AttestFor(who, geode));
            Ok(())
        }

        /// Called by provider to bound dns with geode's ip.
        #[weight = 0]
        pub fn provider_update_geode_dns(origin, geode: T::AccountId, dns: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut geode_use = <Geodes<T>>::get(&geode).ok_or(Error::<T>::InvalidGeode)?;
            ensure!(geode_use.provider == Some(who), Error::<T>::NoRight);
            geode_use.dns = dns;
            <Geodes<T>>::insert(geode.clone(), geode_use);
            Self::deposit_event(RawEvent::DnsUpdate(geode));
            Ok(())
        }

        /// Called by provider to register a geode. The user/attestors/state/provider will be
        /// set automatically regardless of what you set.
        #[weight = 0]
        pub fn provider_register_geode(origin, geode_record: GeodeOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut geode_record = geode_record;
            geode_record.user = None;
            geode_record.attestors = Vec::new();
            geode_record.state = GeodeState::Registered;
            geode_record.provider = Some(who.clone());
            let geode = geode_record.owner.clone();
            ensure!(!<RegisterGeodes<T>>::contains_key(&geode), Error::<T>::AlreadyGeode);
            ensure!(!<Geodes<T>>::contains_key(&geode), Error::<T>::AlreadyGeode);

            let block_number = <frame_system::Module<T>>::block_number();
            let register_geode = RegisterGeode {
                start: block_number.saturated_into::<u64>(),
                geode_record
            };
            <RegisterGeodes<T>>::insert(geode.clone(), register_geode);
            Self::deposit_event(RawEvent::GeodeRegister(who, geode));
            Ok(())
        }

        /// Called by vendor who get the geode from market to update the binary hash
        #[weight = 0]
        pub fn vendor_set_binary(origin, geode: T::AccountId, hash: T::Hash) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut geode_use = <Geodes<T>>::get(&geode).ok_or(Error::<T>::InvalidGeode)?;
            ensure!(geode_use.user == Some(who), Error::<T>::NoRight);
            ensure!(geode_use.state == GeodeState::InWork, Error::<T>::InvalidGeode);
            geode_use.binary = Some(hash);
            <Geodes<T>>::insert(geode.clone(), geode_use);
            Self::deposit_event(RawEvent::SetBinary(geode));
            Ok(())
        }

        /// At every block, check if a geode get enough attestors and turn
        /// it from Registered to Attested. If the geode register Timeout then remove
        /// it.
        fn on_finalize() {
            let now = <frame_system::Module<T>>::block_number().saturated_into::<u64>();
            let mut attested_list = Vec::new();
            let mut timeout_list = Vec::new();
            <RegisterGeodes<T>>::iter().map(|(_key, register_geode)|{
                if register_geode.geode_record.attestors.len() >= ATTESTOR_REQUIRE {
                    attested_list.push(register_geode.geode_record.clone());
                } else if now - register_geode.start > TIMELIMIT {
                    timeout_list.push(register_geode.geode_record.owner.clone());
                }
            }).all(|_| true);
            for mut geode_use in attested_list.clone() {
                <RegisterGeodes<T>>::remove(&geode_use.owner.clone());
                geode_use.state = GeodeState::Attested;
                <Geodes<T>>::insert(geode_use.owner.clone(), geode_use);
            }
            for geode in timeout_list.clone() {
                <RegisterGeodes<T>>::remove(&geode);
            }
            if timeout_list.len() > 0 {
                Self::deposit_event(RawEvent::AttestTimeOut(timeout_list));
            }
        }

        /// Set the min stake request to be an attestor. This should called by "root".
        #[weight = 0]
        pub fn set_att_stake_min(origin, stake: BalanceOf<T>) -> DispatchResult {
            let _who = ensure_root(origin)?;
            <AttStakeMin<T>>::put(stake);
            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    /// Return attestors' url and pubkey list for rpc.
    pub fn attestor_list() -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut res = Vec::new();
        <Attestors<T>>::iter()
            .map(|(_, attestor)| {
                res.push((attestor.url.clone(), attestor.pubkey.clone()));
            })
            .all(|_| true);
        res
    }

    /// Report the misconduct.
    fn report_geode_offence(reporter: T::AccountId, offence: T::AccountId) {
        let offence_geode: Option<GeodeOf<T>> = <Geodes<T>>::get(&offence);

        if offence_geode.is_none() {
            return;
        }

        let geode: GeodeOf<T> = offence_geode.unwrap();
        let provider = geode.provider.unwrap();
        let user = geode.user.unwrap();
        let offence = <T::GeodeOffReport as GeodeReport<T>>::Offence::new(
            offence,
            <frame_system::Module<T>>::block_number().saturated_into::<u64>(),
            provider,
            user,
        );
        T::GeodeOffReport::report(vec![reporter], offence);
    }
}

pub trait Commodity<AccountId> {
    type Value;

    /// Return the geode's provider id.
    fn provider(id: &AccountId) -> Option<AccountId>;
    /// Check if the geode is valid.
    fn contains_key(id: &AccountId) -> bool;
    /// Set the geode's user.
    fn set_user(id: &AccountId, to: Option<AccountId>);
    /// Set the geode's state.
    fn set_geode_state(id: &AccountId, state: GeodeState) -> bool;
}

impl<T: Config> Commodity<T::AccountId> for Module<T> {
    type Value = Geode<T::AccountId, T::Hash>;

    fn provider(id: &T::AccountId) -> Option<T::AccountId> {
        <Geodes<T>>::get(id).and_then(|g| {
            if let Some(provider) = g.provider {
                Some(provider)
            } else {
                None
            }
        })
    }

    fn contains_key(id: &T::AccountId) -> bool {
        <Geodes<T>>::contains_key(id)
    }

    fn set_user(id: &T::AccountId, to: Option<T::AccountId>) {
        <Geodes<T>>::mutate(id, |geode| {
            if let Some(g) = geode {
                (*g).user = to;
            }
        })
    }

    fn set_geode_state(id: &T::AccountId, state: GeodeState) -> bool {
        if !<Geodes<T>>::contains_key(id) {
            return false;
        }
        let mut geode_use = <Geodes<T>>::get(id).unwrap();
        match state.clone() {
            GeodeState::Attested => {
                if geode_use.state != GeodeState::InOrder {
                    return false;
                }
            }
            GeodeState::InOrder => {
                if geode_use.state != GeodeState::Attested {
                    return false;
                }
            }
            GeodeState::InWork => {
                if geode_use.state != GeodeState::InOrder {
                    return false;
                }
            }
            _ => return false,
        }
        geode_use.state = state;
        <Geodes<T>>::insert(id, geode_use);
        true
    }
}
