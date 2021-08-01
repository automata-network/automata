use crate::{Error, ReportType, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn it_works_report_misconduct() {
	new_test_ext().execute_with(|| {
        let attestor_account = 1;
        let geode_account = 2;
        let report_type = ReportType::Challenge;
        let proof: Vec<u8> = vec![];

        register_attestor(attestor_account);
        
        assert_noop!(AttestationModule::report_misconduct(Origin::signed(attestor_account), geode_account, report_type as u8, proof), Error::<Test>::NotAttestingFor);
    });
}