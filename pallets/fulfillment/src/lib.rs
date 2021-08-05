// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
mod property;
mod slash;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    pub use crate::property::{GeodeProperties, GeodeProperty};
    use codec::{Decode, Encode};
    use core::convert::TryInto;
    use dispatch::DispatchResultWithPostInfo;
    use frame_support::pallet_prelude::*;
    use frame_support::{dispatch, ensure, traits::Currency};
    use frame_system::pallet_prelude::*;
    use frame_system::{ensure_root, ensure_signed};
    use sp_runtime::{RuntimeDebug, SaturatedConversion};
    use sp_std::prelude::*;

    pub use crate::slash::{GeodeOffenceTrait, GeodeReport, GeodeReportOffenceHandler};
    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};

    pub const ATTESTOR_REQUIRE: usize = 1;
    pub const TIMELIMIT: u64 = 30;

    /// Geode state
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
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

    impl Default for GeodeState {
        fn default() -> Self {
            GeodeState::Registered
        }
    }

    /// The geode struct shows its status.
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, Default)]
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
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, Default)]
    pub struct Attestor {
        /// Attestor's url, geode will get it and communicate with attestor.
        pub url: Vec<u8>,
        /// Attestor's Secp256r1PublicKey
        pub pubkey: Vec<u8>,
    }

    /// Register Geode struct, used when provider register geode.
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, Default)]
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

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_stake::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type GeodeOffReport: GeodeReport<Self>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// At every block, check if a geode get enough attestors and turn
        /// it from Registered to Attested. If the geode register Timeout then remove
        /// it.
        fn on_finalize(block_number: T::BlockNumber) {
            match TryInto::<u64>::try_into(block_number) {
                Ok(now) => {
                    let mut attested_list = Vec::new();
                    let mut timeout_list = Vec::new();
                    <RegisterGeodes<T>>::iter()
                        .map(|(_key, register_geode)| {
                            if register_geode.geode_record.attestors.len() >= ATTESTOR_REQUIRE {
                                attested_list.push(register_geode.geode_record);
                            } else if now - register_geode.start > 0 {
                                timeout_list.push(register_geode.geode_record.owner);
                            }
                        })
                        .all(|_| true);
                    for mut geode_use in attested_list.clone() {
                        <RegisterGeodes<T>>::remove(&geode_use.owner.clone());
                        geode_use.state = GeodeState::Attested;
                        <Geodes<T>>::insert(geode_use.owner.clone(), geode_use);
                    }
                    for geode in timeout_list.clone() {
                        <RegisterGeodes<T>>::remove(&geode);
                    }
                    if !timeout_list.is_empty() {
                        Self::deposit_event(Event::AttestTimeOut(timeout_list));
                    }
                }
                Err(_) => {}
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {
        /// Somebody report a misconduct. \[reporter, offender\]
        ReportBlame(T::AccountId, T::AccountId),
        /// Attestor registered. \[attestor_id\]
        AttestorRegister(T::AccountId),
        /// Attestor moved. \[attestor_id\]
        AttestorRemove(T::AccountId),
        /// Provider register geode. \[provider_id, geode_id\]
        GeodeRegister(T::AccountId, T::AccountId),
        /// A geode is removed. \[geode_id\]
        GeodeRemove(T::AccountId),
        /// Geode's record updated. \[geode_id\]
        GeodeUpdate(T::AccountId),
        /// User set the binary hash to his geode. \[geode_id\]
        SetBinary(T::AccountId),
        /// Attestor attestor a geode. \[attestor_id, geode_id\]
        AttestFor(T::AccountId, T::AccountId),
        /// Geode's dns updated. \[geode_id\]
        DnsUpdate(T::AccountId),
        /// Attestor's url updated. \[attestor_id\]
        AttestorUpdate(T::AccountId),
        /// Geodes which didn't get enough attestors at limited time after registered.
        /// \[Vec<geode_id>\]
        AttestTimeOut(Vec<T::AccountId>),
    }

    #[pallet::error]
    pub enum Error<T> {
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

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn att_stake_min)]
    pub(super) type AttStakeMin<T: Config> = StorageValue<_, BalanceOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn register_geodes)]
    pub(super) type RegisterGeodes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, RegisterOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn geodes)]
    pub(super) type Geodes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, GeodeOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn attestors)]
    pub(super) type Attestors<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Attestor, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Show that I'm in work. Not in use now.
        #[pallet::weight(0)]

        pub fn im_online(origin: OriginFor<T>, _proof: Vec<u8>) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            // TODO: deal the challenge
            Ok(().into())
        }

        /// Report that somebody did a misconduct. The actual usage is being considered.
        #[pallet::weight(0)]

        pub fn report_misconduct(
            origin: OriginFor<T>,
            geode_id: T::AccountId,
            _proof: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::report_geode_offence(who.clone(), geode_id.clone());
            Self::deposit_event(Event::ReportBlame(who, geode_id));
            Ok(().into())
        }

        /// Called by provider to remove geode .
        /// Return Ok() only when the geode's state is Registered/Attested
        #[pallet::weight(0)]

        pub fn geode_remove(
            origin: OriginFor<T>,
            geode: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            if <Geodes<T>>::contains_key(&geode) {
                let geode_use = <Geodes<T>>::get(&geode);
                ensure!(geode_use.provider == Some(who), Error::<T>::NoRight);
                ensure!(
                    geode_use.state == GeodeState::Attested,
                    Error::<T>::InvalidGeodeState
                );
                <Geodes<T>>::remove(&geode);
            } else if <RegisterGeodes<T>>::contains_key(&geode) {
                let register_geode = <RegisterGeodes<T>>::get(&geode);
                ensure!(
                    register_geode.geode_record.provider == Some(who),
                    Error::<T>::NoRight
                );
                ensure!(
                    register_geode.geode_record.state == GeodeState::Registered,
                    Error::<T>::InvalidGeodeState
                );
                <RegisterGeodes<T>>::remove(&geode);
            } else {
                return Err(Error::<T>::InvalidGeode.into());
            }

            Self::deposit_event(Event::GeodeRemove(geode));
            Ok(().into())
        }

        /// Called by provider to update geode's record
        ///
        /// #note
        ///
        /// The user/state/attestor should not be managed by provider,
        /// and change the provider is dangerous, once you change the provider,
        /// you won't own the control of the geode.
        #[pallet::weight(0)]

        pub fn geode_update(
            origin: OriginFor<T>,
            geode: T::AccountId,
            record: GeodeOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let geode_use = <Geodes<T>>::get(&geode);
            ensure!(geode_use.provider == Some(who), Error::<T>::NoRight);
            ensure!(
                geode_use.state != GeodeState::InWork,
                Error::<T>::GeodeInWork
            );
            let mut record = record;
            record.attestors = geode_use.attestors;
            record.state = geode_use.state;
            record.user = geode_use.user;
            <Geodes<T>>::insert(geode.clone(), record);
            Self::deposit_event(Event::GeodeUpdate(geode));
            Ok(().into())
        }

        /// Register as an attestor.
        #[pallet::weight(0)]

        pub fn attestor_register(
            origin: OriginFor<T>,
            url: Vec<u8>,
            pubkey: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let limit = <AttStakeMin<T>>::get().ok_or(Error::<T>::InvalidAttestor)?;
            <pallet_stake::Module<T>>::enough_stake(&who, limit)?;
            let attestor = Attestor { url, pubkey };
            <Attestors<T>>::insert(who.clone(), attestor);
            Self::deposit_event(Event::AttestorRegister(who));
            Ok(().into())
        }

        /// Remove self from attestors.
        ///
        ///  #note
        ///
        /// Currently, we use reliable attestor which would not do misconducts.
        /// This function should be called when the attestor is not serving for any geode.
        #[pallet::weight(0)]

        pub fn attestor_remove(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                <Attestors<T>>::contains_key(&who),
                Error::<T>::InvalidAttestor
            );
            <Attestors<T>>::remove(&who);
            Self::deposit_event(Event::AttestorRemove(who));
            Ok(().into())
        }

        /// Called by attestor to update its url.
        #[pallet::weight(0)]

        pub fn attestor_update(origin: OriginFor<T>, data: Vec<u8>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let mut attestor = <Attestors<T>>::get(&who);
            attestor.url = data;
            <Attestors<T>>::insert(who.clone(), attestor);
            Self::deposit_event(Event::AttestorUpdate(who));
            Ok(().into())
        }

        /// Called by attestor to attest Geode.
        #[pallet::weight(0)]

        pub fn attestor_attest_geode(
            origin: OriginFor<T>,
            geode: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                <Attestors<T>>::contains_key(&who),
                Error::<T>::InvalidAttestor
            );

            let mut register_geode = <RegisterGeodes<T>>::get(&geode);
            for att in register_geode.geode_record.attestors.clone() {
                ensure!(att != who, Error::<T>::AlreadyAttestFor);
            }
            register_geode.geode_record.attestors.push(who.clone());
            <RegisterGeodes<T>>::insert(geode.clone(), register_geode);

            Self::deposit_event(Event::AttestFor(who, geode));
            Ok(().into())
        }

        /// Called by provider to bound dns with geode's ip.
        #[pallet::weight(0)]

        pub fn provider_update_geode_dns(
            origin: OriginFor<T>,
            geode: T::AccountId,
            dns: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let mut geode_use = <Geodes<T>>::get(&geode);
            ensure!(geode_use.provider == Some(who), Error::<T>::NoRight);
            geode_use.dns = dns;
            <Geodes<T>>::insert(geode.clone(), geode_use);
            Self::deposit_event(Event::DnsUpdate(geode));
            Ok(().into())
        }

        /// Called by provider to register a geode. The user/attestors/state/provider will be
        /// set automatically regardless of what you set.
        #[pallet::weight(0)]

        pub fn provider_register_geode(
            origin: OriginFor<T>,
            geode_record: GeodeOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let mut geode_record = geode_record;
            geode_record.user = None;
            geode_record.attestors = Vec::new();
            geode_record.state = GeodeState::Registered;
            geode_record.provider = Some(who.clone());
            let geode = geode_record.owner.clone();
            ensure!(
                !<RegisterGeodes<T>>::contains_key(&geode),
                Error::<T>::AlreadyGeode
            );
            ensure!(!<Geodes<T>>::contains_key(&geode), Error::<T>::AlreadyGeode);

            let block_number = <frame_system::Module<T>>::block_number();
            let register_geode = RegisterGeode {
                start: block_number.saturated_into::<u64>(),
                geode_record,
            };
            <RegisterGeodes<T>>::insert(geode.clone(), register_geode);
            Self::deposit_event(Event::GeodeRegister(who, geode));
            Ok(().into())
        }

        /// Called by vendor who get the geode from market to update the binary hash
        #[pallet::weight(0)]

        pub fn vendor_set_binary(
            origin: OriginFor<T>,
            geode: T::AccountId,
            hash: T::Hash,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let mut geode_use = <Geodes<T>>::get(&geode);
            ensure!(geode_use.user == Some(who), Error::<T>::NoRight);
            ensure!(
                geode_use.state == GeodeState::InWork,
                Error::<T>::InvalidGeode
            );
            geode_use.binary = Some(hash);
            <Geodes<T>>::insert(geode.clone(), geode_use);
            Self::deposit_event(Event::SetBinary(geode));
            Ok(().into())
        }

        /// Set the min stake request to be an attestor. This should called by "root".
        #[pallet::weight(0)]

        pub fn set_att_stake_min(
            origin: OriginFor<T>,
            stake: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_root(origin)?;
            <AttStakeMin<T>>::put(stake);
            Ok(().into())
        }
    }

    /// ledger
    impl<T: Config> Pallet<T> {
        /// Return attestors' url and pubkey list for rpc.
        pub fn attestor_list() -> Vec<(Vec<u8>, Vec<u8>)> {
            let mut res = Vec::new();
            <Attestors<T>>::iter()
                .map(|(_, attestor)| {
                    res.push((attestor.url.clone(), attestor.pubkey));
                })
                .all(|_| true);
            res
        }

        pub fn registered_geodes() -> Vec<GeodeOf<T>> {
            let mut res = Vec::new();
            <RegisterGeodes<T>>::iter()
                .map(|(_, geode)| {
                    res.push(geode.geode_record);
                })
                .all(|_| true);
            res
        }

        /// Report the misconduct.
        fn report_geode_offence(reporter: T::AccountId, offence: T::AccountId) {
            let geode: GeodeOf<T> = <Geodes<T>>::get(&offence);
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
        fn set_user(id: &AccountId, to: AccountId);
        /// Set the geode's state.
        fn set_geode_state(id: &AccountId, state: GeodeState) -> bool;
    }

    impl<T: Config> Commodity<T::AccountId> for Pallet<T> {
        type Value = Geode<T::AccountId, T::Hash>;

        fn provider(id: &T::AccountId) -> Option<T::AccountId> {
            <Geodes<T>>::get(id).provider
        }

        fn contains_key(id: &T::AccountId) -> bool {
            <Geodes<T>>::contains_key(id)
        }

        fn set_user(id: &T::AccountId, to: T::AccountId) {
            <Geodes<T>>::mutate(id, |geode| geode.user = Some(to))
        }

        fn set_geode_state(id: &T::AccountId, state: GeodeState) -> bool {
            if !<Geodes<T>>::contains_key(id) {
                return false;
            }
            let mut geode_use = <Geodes<T>>::get(id);
            match state {
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
}
