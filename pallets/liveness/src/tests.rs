use crate::{mock::*, Error, ReportType};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_attestor_attest_geode() {
    new_test_ext().execute_with(|| {
        let attestor_account = 1;
        let geode_account = 2;

        register_attestor(attestor_account);
        provider_register_geode(attestor_account, geode_account);

        assert_ok!(LivenessModule::attestor_attest_geode(
            Origin::signed(attestor_account),
            geode_account
        ));
    });
}

#[test]
fn it_attestor_attest_geode_invalid_attestor() {
    new_test_ext().execute_with(|| {
        let attestor_account = 1;
        let geode_account = 2;

        // attestor not registered
        assert_noop!(
            LivenessModule::attestor_attest_geode(Origin::signed(attestor_account), geode_account),
            pallet_attestor::Error::<Test>::InvalidAttestor
        );
    });
}

#[test]
fn it_attestor_attest_geode_already_attestor() {
    new_test_ext().execute_with(|| {
        let attestor_account = 1;
        let geode_account = 2;

        register_attestor(attestor_account);
        provider_register_geode(attestor_account, geode_account);

        assert_ok!(LivenessModule::attestor_attest_geode(
            Origin::signed(attestor_account),
            geode_account
        ));

        // readly registered before
        assert_noop!(
            LivenessModule::attestor_attest_geode(Origin::signed(attestor_account), geode_account),
            Error::<Test>::AlreadyAttestFor
        );
    });
}

#[test]
fn it_attestor_attest_geode_invalid_geode() {
    new_test_ext().execute_with(|| {
        let attestor_account = 1;
        let geode_account = 2;

        register_attestor(attestor_account);

        // geode not registered before
        assert_noop!(
            LivenessModule::attestor_attest_geode(Origin::signed(attestor_account), geode_account),
            pallet_geode::Error::<Test>::InvalidGeode
        );
    });
}

#[test]
fn it_works_report_misconduct() {
    new_test_ext().execute_with(|| {
        let attestor_account = 1;
        let geode_account = 2;
        let report_type = ReportType::Challenge;
        let proof: Vec<u8> = vec![];

        register_attestor(attestor_account);
        provider_register_geode(attestor_account, geode_account);

        assert_ok!(LivenessModule::attestor_attest_geode(
            Origin::signed(attestor_account),
            geode_account
        ));

        // report misconduct successfully
        assert_ok!(LivenessModule::report_misconduct(
            Origin::signed(attestor_account),
            geode_account,
            report_type as u8,
            proof
        ));
    });
}

#[test]
fn it_report_misconduct_invalid_attestor() {
    new_test_ext().execute_with(|| {
        let attestor_account = 1;
        let geode_account = 2;
        let report_type = ReportType::Challenge;
        let proof: Vec<u8> = vec![];

        // attestor not registered
        assert_noop!(
            LivenessModule::report_misconduct(
                Origin::signed(attestor_account),
                geode_account,
                report_type as u8,
                proof
            ),
            pallet_attestor::Error::<Test>::InvalidAttestor
        );
    });
}

#[test]
fn it_report_misconduct_not_attesting_for() {
    new_test_ext().execute_with(|| {
        let attestor_account = 1;
        let geode_account = 2;
        let report_type = ReportType::Challenge;
        let proof: Vec<u8> = vec![];
        register_attestor(attestor_account);

        // attestor not for the geode
        assert_noop!(
            LivenessModule::report_misconduct(
                Origin::signed(attestor_account),
                geode_account,
                report_type as u8,
                proof
            ),
            Error::<Test>::NotAttestingFor
        );
    });
}

#[test]
fn it_report_misconduct_invalid_report_type() {
    new_test_ext().execute_with(|| {
        let attestor_account = 1;
        let geode_account = 2;
        // wrong report type
        let report_type = 100_u8;
        let proof: Vec<u8> = vec![];

        register_attestor(attestor_account);
        provider_register_geode(attestor_account, geode_account);

        assert_ok!(LivenessModule::attestor_attest_geode(
            Origin::signed(attestor_account),
            geode_account
        ));

        // report type is wrong
        assert_noop!(
            LivenessModule::report_misconduct(
                Origin::signed(attestor_account),
                geode_account,
                report_type,
                proof
            ),
            Error::<Test>::InvalidReportType
        );
    });
}
