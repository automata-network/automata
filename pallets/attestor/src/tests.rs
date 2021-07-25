use crate::{Error, Attestor, mock::*};
use frame_support::{assert_ok, assert_noop};

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
		assert_ok!(AttestorModule::attestor_register(Origin::signed(attestor_account), url.clone(), pubkey.clone()));
		let data = AttestorModule::attestors(&attestor_account);
		
		// check the data inserted is correct
		assert_eq!(data, Attestor{
			url: url,
			pubkey: pubkey,
		});

		// check the event emit
		assert_eq!(
			events(),
			[
				Event::pallet_balances(pallet_balances::Event::Reserved(attestor_account, min_stake)),
				Event::attestor(crate::Event::AttestorRegister(attestor_account)),
			]
		);

		// check the balance
		let attestor_balance = Balances::free_balance(attestor_account);
		assert_eq!(attestor_balance, INIT_BALANCE - min_stake);
	});
}

