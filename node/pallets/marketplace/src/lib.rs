// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use dispatch::DispatchResult;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, traits::Currency,
    StorageMap,
};
use frame_system::ensure_signed;
use fulfillment::{self, Commodity, GeodeState};
use primitives::BlockNumber;
use sp_runtime::{RuntimeDebug, SaturatedConversion};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The MarketOrder struct, record the order's messages.
#[derive(Default, PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct MarketOrder<AccountId, Balance> {
    /// Seller's id. (provider)
    pub seller: AccountId,
    /// Geode's id
    pub geode: AccountId,
    /// The price for one block.
    pub price: Balance,
    /// Max duration for sell.
    pub duration: BlockNumber,
}

/// Record for a ture deal.
#[derive(Default, PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct MarketDeal<AccountId, Balance> {
    /// Seller's id.
    pub seller: AccountId,
    /// Buyer's id.
    pub buyer: AccountId,
    /// Geode;s id.
    pub geode: AccountId,
    /// The deal's start block number.
    pub created_on: BlockNumber,
    /// Price for one block.
    pub price: Balance,
    /// Actual deal duration.
    pub duration: BlockNumber,
}

impl<AccountId, Balance> MarketDeal<AccountId, Balance> {
    /// Check if the deal is overed.
    pub fn is_expire(&self, now: BlockNumber) -> bool {
        self.created_on + self.duration >= now
    }
}

pub type OrderId<T> = <T as frame_system::Config>::AccountId;
pub type DealId<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as pallet_stake::Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;
pub type MarketOrderOf<T> = MarketOrder<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

pub type MarketDealOf<T> = MarketDeal<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

pub type GeodeId<T> = <T as frame_system::Config>::AccountId;

pub trait Config: frame_system::Config + pallet_stake::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

    type Commodity: Commodity<Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Config> as Marketplace {
        /// Record of orders in the market.
        Orders get(fn orders): map hasher(twox_64_concat) OrderId<T> => Option<MarketOrderOf<T>>;
        /// Record of deals.
        Deals get(fn deals): map hasher(twox_64_concat) DealId<T> => MarketDealOf<T>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        Balance = BalanceOf<T>,
        OrderId = <T as frame_system::Config>::AccountId,
        DealId = <T as frame_system::Config>::AccountId,
    {
        /// User registered.
        UserAdded(AccountId, Balance),
        /// User pledge some balances.
        UserPledged(AccountId, Balance),
        /// Provider put geode into market for sell.
        PutOrder(AccountId, OrderId),
        /// Provider cancel the order.
        CancelOrder(OrderId),
        /// A deal was matched.
        MakeDeal(DealId),
        /// Stop the deal.
        ExpireDeal(DealId),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Not the provider of geode and have no right to handle it.
        NotProvider,
        /// Use an invalid geode id.
        InvalidGeodeId,
        /// Use an invalid order id.
        InvalidOrderId,
        /// Use an invalid deal id.
        InvalidDealId,
        /// The deal is over time and didn't expired.
        UnexpiredDeal,
        /// The geode is in a state that you can't do this now.
        InvalidGeodeState,
        /// Geode have no provider, this should happen.
        NoProvider,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Register as a user.
        #[weight = 0]
        pub fn register(origin, amount: BalanceOf<T>) -> DispatchResult {
            let actor = ensure_signed(origin)?;

            <pallet_stake::Module<T>>::register(actor.clone(),amount)?;

            Self::deposit_event(RawEvent::UserAdded(actor, amount));

            Ok(())
        }

        /// Pledge some balances to trade in the market.
        #[weight = 0]
        pub fn pledge(origin, amount: BalanceOf<T>) -> DispatchResult {
            let actor = ensure_signed(origin)?;

             <pallet_stake::Module<T>>::pledge(actor.clone(),amount)?;

            Self::deposit_event(RawEvent::UserPledged(actor, amount));

            Ok(())
        }

        /// User withdraw balances.
        #[weight = 0]
        pub fn withdraw(origin, amount: BalanceOf<T>) -> DispatchResult {
            let actor = ensure_signed(origin)?;

            <pallet_stake::Module<T>>::withdraw(actor.clone(),amount)?;

            Self::deposit_event(RawEvent::UserPledged(actor, amount));

            Ok(())
        }

        /// Provider put its geode into market for sell.
        #[weight = 0]
        pub fn put_order(origin, geode_id: GeodeId<T>, price: BalanceOf<T>, duration: BlockNumber) -> DispatchResult {
            let actor = ensure_signed(origin)?;

            <pallet_stake::Module<T>>::is_not_registered(&actor)?;
            ensure!(T::Commodity::contains_key(&geode_id), Error::<T>::InvalidGeodeId);
            let provider = T::Commodity::provider(&geode_id).ok_or(Error::<T>::NoProvider)?;
            ensure!(provider == actor, Error::<T>::NotProvider);

            let pledge = price * duration.into();
            <pallet_stake::Module<T>>::enough_stake_put_order(&actor, pledge)?;

            ensure!(T::Commodity::set_geode_state(&geode_id, GeodeState::InOrder), Error::<T>::InvalidGeodeState);
            let order = MarketOrder {
                seller: actor.clone(),
                geode: geode_id.clone(),
                price: price,
                duration: duration,
            };
            let order_id = geode_id.clone();

            <Orders<T>>::insert(&order_id, order);

            // update ledger
            <pallet_stake::Module<T>>::update_put_order(&actor, pledge, order_id)?;

            Self::deposit_event(RawEvent::PutOrder(actor, geode_id));
            Ok(())
        }

        /// Provider cancel the order before the geode is sold.
        #[weight = 0]
        pub fn cancel_order(origin, order_id: OrderId<T>) -> DispatchResult {
            let actor = ensure_signed(origin)?;

            <pallet_stake::Module<T>>::is_not_registered(&actor)?;

            ensure!(<Orders<T>>::contains_key(&order_id), Error::<T>::InvalidOrderId);
            let order = <Orders<T>>::get(&order_id).unwrap();
            ensure!(order.seller == actor, Error::<T>::NotProvider);
            ensure!(T::Commodity::set_geode_state(&order.geode, GeodeState::Attested), Error::<T>::InvalidGeodeState);
            Self::remove_order(&actor, &order_id);

            Self::deposit_event(RawEvent::CancelOrder(order_id));

            Ok(())
        }

        /// User match the order.
        /// If user have enough balances pledged, he will match the order and get the geode's use right.
        #[weight = 0]
        pub fn match_order(origin, order_id: OrderId<T>, duration: BlockNumber) -> DispatchResult {
            let actor = ensure_signed(origin)?;

            <pallet_stake::Module<T>>::is_not_registered(&actor)?;
            ensure!(<Orders<T>>::contains_key(&order_id), Error::<T>::InvalidOrderId);
            let order = <Orders<T>>::get(&order_id).unwrap();
            let seller = order.seller.clone();
            let cost = order.price * duration.into();

            <pallet_stake::Module<T>>::enough_stake(&actor, cost)?;

            // convert order to deal
            let deal = MarketDeal {
                seller: order.seller,
                buyer: actor.clone(),
                geode: order.geode.clone(),
                created_on: <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>(),
                price: order.price,
                duration: duration,
            };
            let deal_id = order_id.clone();
            ensure!(T::Commodity::set_geode_state(&order.geode, GeodeState::InWork), Error::<T>::InvalidGeodeState);
            T::Commodity::set_user(&order.geode, Some(actor.clone()));
            <pallet_stake::Module<T>>::match_order_exchange(actor.clone(), seller, order_id.clone(), duration,cost)?;

            <Deals<T>>::insert(&deal_id, deal);
            Self::deposit_event(RawEvent::MakeDeal(order_id));
            Ok(())
        }

        /// Stop the deal before it over's naturally.
        #[weight = 0]
        pub fn expire_deal(origin, deal_id: DealId<T>) -> DispatchResult {
            let actor = ensure_signed(origin)?;
            <pallet_stake::Module<T>>::is_registered(&actor)?;
            ensure!(<Deals<T>>::contains_key(&deal_id), Error::<T>::InvalidDealId);

            // release to active
            let deal = <Deals<T>>::get(&deal_id);
            let now = <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>();
            ensure!(deal.is_expire(now), Error::<T>::UnexpiredDeal);
            ensure!(T::Commodity::set_geode_state(&deal.geode, GeodeState::Attested), Error::<T>::InvalidGeodeState);
            T::Commodity::set_user(&deal.geode, None);
            <pallet_stake::Module<T>>::remove_order_or_expire_deal(deal.seller,deal_id.clone())?;

            Self::deposit_event(RawEvent::ExpireDeal(deal_id));

            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    fn remove_order(actor: &T::AccountId, order_id: &OrderId<T>) {
        let _ =
            <pallet_stake::Module<T>>::remove_order_or_expire_deal(actor.clone(), order_id.clone());
        <Orders<T>>::remove(order_id);
    }
}
