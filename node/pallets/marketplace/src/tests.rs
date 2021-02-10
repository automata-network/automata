// SPDX-License-Identifier: Apache-2.0

use crate::mock::TestEvent;
use crate::sp_api_hidden_includes_decl_storage::hidden_include::traits::OnFinalize;
use crate::RawEvent;
use crate::{mock::*, Error};
use frame_support::{
    assert_err, assert_err_ignore_postinfo, assert_noop, assert_ok,
    dispatch::{DispatchError, DispatchErrorWithPostInfo, Dispatchable},
    impl_outer_dispatch, impl_outer_event, impl_outer_origin, parameter_types, storage,
    traits::Filter,
    weights::{Pays, Weight},
};
use fulfillment::{GeodeOf, GeodeProperties, GeodeState, RawEvent as geodeRawEvent};
use sp_core::H256;
use sp_std::prelude::*;

fn create_geode_device<T: fulfillment::Config>(owner: T::AccountId) -> GeodeOf<T> {
    let geode_state = GeodeState::Registered;
    let str_props = r#"
        {
            "Instance": "DC1s v2",
            "Core": "1",
            "RAM": "4 GiB",
            "Temporary storage": "50GiB"
        }
        "#;
    let _target_props = GeodeProperties::from_json(&str_props).unwrap();
    let device = GeodeOf::<T> {
        owner: owner,
        user: None,                               //Option<AccountId>,
        provider: None,                           //Option<AccountId>,
        ip: vec![0u8],                            //Vec<u8>,
        dns: vec![0u8],                           //Vec<u8>,
        props: None,                              //Option<GeodeProperties>,
        binary: None,                             //Option<Hash>,
        attestors: vec![T::AccountId::default()], //Vec<AccountId>,
        state: geode_state,
    };
    device
}

fn last_event() -> TestEvent {
    System::events()
        .pop()
        .map(|e| e.event)
        .expect("Event expected")
}

fn get_orderid_from_event() -> AccountId {
    match marketpalce_events().last().unwrap() {
        RawEvent::PutOrder(x, y) => {
            return *y;
        }
        _ => {
            return 0;
        }
    }
}

fn get_dealid_from_event() -> AccountId {
    match marketpalce_events().last().unwrap() {
        RawEvent::MakeDeal(dealid) => {
            return *dealid;
        }
        _ => {
            return 0;
        }
    }
}

fn geode_regist_id_from_event() -> AccountId {
    match geode_events().last().unwrap() {
        geodeRawEvent::GeodeRegister(_, id) => {
            return *id;
        }
        _ => {
            return 0;
        }
    }
}

#[test]
fn shoule_register() {
    new_test_ext().execute_with(|| {
        assert_ok!(Marketplace::register(Origin::signed(1), 50));
        assert_eq!(Stake::users(1).total, 50);
        assert_eq!(Stake::users(1).active, 50);

        assert_err!(
            Marketplace::register(Origin::signed(1), 50),
            pallet_stake::Error::<Test>::AlreadyRegistered
        );
    });
}

#[test]
fn pledge_test() {
    new_test_ext().execute_with(|| {
        assert_ok!(Marketplace::register(Origin::signed(1), 50));
        assert_eq!(Stake::users(1).total, 50);
        assert_eq!(Stake::users(1).active, 50);

        assert_ok!(Marketplace::pledge(Origin::signed(1), 50));
        assert_eq!(Stake::users(1).total, 100);
        assert_eq!(Stake::users(1).active, 100);

        assert_err!(
            Marketplace::pledge(Origin::signed(2), 50),
            pallet_stake::Error::<Test>::NotRegistered
        );
    });
}

#[test]
fn withdraw_test() {
    new_test_ext().execute_with(|| {
        assert_ok!(Marketplace::register(Origin::signed(1), 50));
        assert_eq!(Stake::users(1).total, 50);
        assert_eq!(Stake::users(1).active, 50);

        assert_ok!(Marketplace::withdraw(Origin::signed(1), 50));
        assert_eq!(Stake::users(1).total, 0);
        assert_eq!(Stake::users(1).active, 0);

        assert_err!(
            Marketplace::withdraw(Origin::signed(2), 50),
            pallet_stake::Error::<Test>::NotRegistered
        );
    });
}

#[test]
fn install_test() {
    new_test_ext().execute_with(|| {
        assert_ok!(Marketplace::register(Origin::signed(1), 50));
        assert_eq!(Stake::users(1).total, 50);
        assert_eq!(Stake::users(1).active, 50);

        let attestation = vec![0u8];
        let device = create_geode_device::<Test>(1);
        assert_ok!(Geode::provider_register_geode(Origin::signed(1), device));
        let event = last_event();
        assert_eq!(
            event,
            TestEvent::fulfillment(geodeRawEvent::GeodeRegister(1, 1))
        );
    });
}

#[test]
fn uninstall_test() {
    new_test_ext().execute_with(|| {
        assert_ok!(Marketplace::register(Origin::signed(1), 50));
        assert_eq!(Stake::users(1).total, 50);
        assert_eq!(Stake::users(1).active, 50);

        let device = create_geode_device::<Test>(1);
        assert_ok!(Geode::provider_register_geode(Origin::signed(1), device));
        let geoid = geode_regist_id_from_event();

        assert_ok!(Geode::geode_remove(Origin::signed(1), geoid));
        assert_eq!(Geode::geodes(geoid), None);
    });
}

#[test]
fn order_test() {
    new_test_ext().execute_with(|| {
        assert_ok!(Marketplace::register(Origin::signed(1), 50));
        assert_eq!(Stake::users(1).total, 50);
        assert_eq!(Stake::users(1).active, 50);

        let device = create_geode_device::<Test>(1);
        assert_ok!(Geode::provider_register_geode(Origin::signed(1), device));
        let geoid = geode_regist_id_from_event();

        assert_ok!(Geode::attestor_register(
            Origin::signed(2),
            vec![0u8],
            Vec::new()
        ));
        assert_ok!(Geode::attestor_attest_geode(Origin::signed(2), geoid));
        Geode::on_finalize(1);
        assert_eq!(
            Geode::geodes(geoid),
            Some(GeodeOf::<Test> {
                owner: 1,
                user: None,
                provider: Some(1),
                ip: vec![0],
                dns: vec![0],
                props: None,
                binary: None,
                attestors: vec![2],
                state: GeodeState::Attested
            })
        );
        //TODO:: Registed -> Attested     attestors only 2

        println!("{:?}", geoid);
        assert_ok!(Marketplace::put_order(Origin::signed(1), geoid, 5, 5));
        let orderid = get_orderid_from_event();

        assert_ok!(Marketplace::cancel_order(Origin::signed(1), orderid));

        assert_ok!(Marketplace::put_order(Origin::signed(1), geoid, 5, 5));
        let _orderid_to_match = get_orderid_from_event();

        assert_ok!(Marketplace::register(Origin::signed(2), 50));
        assert_ok!(Marketplace::match_order(Origin::signed(2), geoid, 5));
        let dealid = get_dealid_from_event();
        assert_eq!(Marketplace::deals(dealid).geode, geoid);
    });
}
