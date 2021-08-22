use crate::{mock::*, Attestor};
use frame_support::assert_ok;
use frame_system::pallet_prelude::*;
use hex_literal::hex;
use primitives::AccountId;

#[test]
fn it_works_for_attestor_register() {
    new_test_ext().execute_with(|| {
        let url = vec![1];
        let pubkey = vec![2];
        let min_stake = 100;
        let attestor_account = 1;

        // set the min stake balance
        assert_ok!(AttestorModule::set_att_stake_min(Origin::root(), min_stake));

        // successfully call register
        assert_ok!(AttestorModule::attestor_register(
            Origin::signed(attestor_account),
            url.clone(),
            pubkey.clone()
        ));
        let data = AttestorModule::attestors(&attestor_account);

        // check the data inserted is correct
        assert_eq!(
            data,
            Attestor {
                url: url,
                pubkey: pubkey,
                geodes: Default::default(),
            }
        );

        // check the event emit
        assert_eq!(
            events(),
            [
                Event::pallet_balances(pallet_balances::Event::Reserved(
                    attestor_account,
                    min_stake
                )),
                Event::attestor(crate::Event::AttestorRegister(attestor_account)),
            ]
        );

        // check the balance
        let attestor_balance = Balances::free_balance(attestor_account);
        assert_eq!(attestor_balance, INIT_BALANCE - min_stake);
    });
}

#[test]
fn get_ss58_address_from_pubkey() {
    new_test_ext().execute_with(|| {
        let binary: [u8; 32] =
            hex!["be7604b40c9eabbfdf62f2041a8b40e160799919e06c6395cda43083c9453b7b"].into();
        let addr: AccountId = binary.into();
        println!("{:?}", addr);
    });
}

#[test]
fn it_works_for_attestor_remove() {
    new_test_ext().execute_with(|| {
        let url = vec![1];
        let pubkey = vec![2];
        let min_stake = 100;
        let attestor_account = 1;

        // set the min stake balance
        assert_ok!(AttestorModule::set_att_stake_min(Origin::root(), min_stake));

        // successfully call register
        assert_ok!(AttestorModule::attestor_register(
            Origin::signed(attestor_account),
            url.clone(),
            pubkey.clone()
        ));

        // remove old events
        events();

        // call remove
        AttestorModule::attestor_remove(ensure_signed(Origin::signed(attestor_account)).unwrap());
        let data = AttestorModule::attestors(&attestor_account);

        // check the data after remove
        assert_eq!(
            data,
            Attestor {
                url: vec![],
                pubkey: vec![],
                geodes: Default::default(),
            }
        );

        // check the event emit
        assert_eq!(
            events(),
            [Event::attestor(crate::Event::AttestorRemove(
                attestor_account
            )),]
        );
    });
}

#[test]
fn it_works_for_attestor_update() {
    new_test_ext().execute_with(|| {
        let url = vec![1];
        let pubkey = vec![2];
        let min_stake = 100;
        let attestor_account = 1;

        // set the min stake balance
        assert_ok!(AttestorModule::set_att_stake_min(Origin::root(), min_stake));

        // successfully call register
        assert_ok!(AttestorModule::attestor_register(
            Origin::signed(attestor_account),
            url.clone(),
            pubkey.clone()
        ));

        let data = AttestorModule::attestors(&attestor_account);

        // check the data inserted is correct
        assert_eq!(
            data,
            Attestor {
                url: url,
                pubkey: pubkey.clone(),
                geodes: Default::default(),
            }
        );

        // remove old events
        events();

        // successfully call update
        let new_url = vec![3];
        assert_ok!(AttestorModule::attestor_update(
            Origin::signed(attestor_account),
            new_url.clone()
        ));
        let data = AttestorModule::attestors(&attestor_account);

        // check the data after remove
        assert_eq!(
            data,
            Attestor {
                url: new_url,
                pubkey: pubkey,
                geodes: Default::default(),
            }
        );

        // check the event emit
        assert_eq!(
            events(),
            [Event::attestor(crate::Event::AttestorUpdate(
                attestor_account
            )),]
        );
    });
}

#[test]
fn it_works_for_set_att_stake_min() {
    new_test_ext().execute_with(|| {
        let min_stake = 100;

        assert_ok!(AttestorModule::set_att_stake_min(Origin::root(), min_stake));

        // test the value is correct
        assert_eq!(AttestorModule::att_stake_min(), min_stake);
    });
}
