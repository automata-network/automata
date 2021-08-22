use crate::{mock::*, Attestor};
use frame_support::assert_ok;

#[test]
fn it_works_for_attestor_register() {
    new_test_ext().execute_with(|| {
        let url = vec![1];
        let pubkey = vec![2];
        let min_stake = 100;
        let attestor_account = 1;

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
                Event::attestor(crate::Event::AttestorRegister(attestor_account)),
            ]
        );
    });
}

#[test]
fn it_works_for_attestor_remove() {
    new_test_ext().execute_with(|| {
        let url = vec![1];
        let pubkey = vec![2];
        let min_stake = 100;
        let attestor_account = 1;

        // successfully call register
        assert_ok!(AttestorModule::attestor_register(
            Origin::signed(attestor_account),
            url.clone(),
            pubkey.clone()
        ));

        // remove old events
        events();

        // successfully call remove
        assert_ok!(AttestorModule::attestor_remove(Origin::signed(
            attestor_account
        )));
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
