use crate::mock::*;
use frame_support::assert_ok;
use sp_core::H160;

#[test]
fn it_works_transfer_to_evm_account() {
    new_test_ext().execute_with(|| {
        let sender_account = 1;
        let receiver_account = H160::zero();
        let transferred_account = TransferModule::evm_address_to_account_id(receiver_account);

        // successfully call transfer_to_evm_account
        assert_ok!(TransferModule::transfer_to_evm_account(
            Origin::signed(sender_account),
            receiver_account,
            100_u32.into()
        ));

        assert_eq!(Balances::free_balance(sender_account), INIT_BALANCE - 100);
        assert_eq!(Balances::free_balance(transferred_account), 100);
    })
}
