use crate::{Error, mock::*};
use crate as pallet_geode;
use frame_support::{assert_ok, assert_noop};

#[test]
fn it_works_provider_register_geode(){
	new_test_ext().execute_with(|| {
        let attestor_account = 1;
        let geode_account = 2;
        let geode_id = 3;
        let provider = 4;

        register_attestor(attestor_account);
        
        let geode: pallet_geode::Geode::< <Test as frame_system::Config>::AccountId, <Test as frame_system::Config>::Hash > = pallet_geode::Geode {
            id: geode_id,
            provider: provider,
            order: None,
            ip: vec![],
            dns: vec![],
            props: Default::default(),
            state: Default::default(),
            promise: Default::default(),
        };
    
        assert_ok!(GeodeModule::provider_register_geode(Origin::signed(provider), geode));

    });
}


#[test]
fn it_works_geode_remove(){
	new_test_ext().execute_with(|| {
        let attestor_account = 1;
        let geode_account = 2;
        let geode_id = 3;
        let provider = 4;

        register_attestor(attestor_account);
        
        let geode: pallet_geode::Geode::< <Test as frame_system::Config>::AccountId, <Test as frame_system::Config>::Hash > = pallet_geode::Geode {
            id: geode_id,
            provider: provider,
            order: None,
            ip: vec![],
            dns: vec![],
            props: Default::default(),
            state: Default::default(),
            promise: Default::default(),
        };
    
        assert_ok!(GeodeModule::provider_register_geode(Origin::signed(provider), geode));

        assert_ok!(GeodeModule::geode_remove(Origin::signed(provider), geode_id));
    });
}


#[test]
fn it_works_update_geode_props(){
	new_test_ext().execute_with(|| {
        let attestor_account = 1;
        let geode_account = 2;
        let geode_id = 3;
        let provider = 4;

        register_attestor(attestor_account);
        
        let geode: pallet_geode::Geode::< <Test as frame_system::Config>::AccountId, <Test as frame_system::Config>::Hash > = pallet_geode::Geode {
            id: geode_id,
            provider: provider,
            order: None,
            ip: vec![],
            dns: vec![],
            props: Default::default(),
            state: Default::default(),
            promise: Default::default(),
        };
    
        assert_ok!(GeodeModule::provider_register_geode(Origin::signed(provider), geode));

        let prop_name = vec![0x79_u8, 0x70_u8];
        let prop_value = vec![1_u8];
        assert_ok!(GeodeModule::update_geode_props(Origin::signed(provider), geode_id, prop_name, prop_value));
    });
}


#[test]
fn it_works_update_geode_dns(){
	new_test_ext().execute_with(|| {
        let attestor_account = 1;
        let geode_account = 2;
        let geode_id = 3;
        let provider = 4;

        register_attestor(attestor_account);
        
        let geode: pallet_geode::Geode::< <Test as frame_system::Config>::AccountId, <Test as frame_system::Config>::Hash > = pallet_geode::Geode {
            id: geode_id,
            provider: provider,
            order: None,
            ip: vec![],
            dns: vec![],
            props: Default::default(),
            state: Default::default(),
            promise: Default::default(),
        };
    
        assert_ok!(GeodeModule::provider_register_geode(Origin::signed(provider), geode));

        let dns = vec![1_u8];
        assert_ok!(GeodeModule::update_geode_dns(Origin::signed(provider), geode_id, dns));
    });
}
