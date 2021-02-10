// SPDX-License-Identifier: Apache-2.0

use crate::mock::TestEvent;
use crate::RawEvent;
use crate::{mock::*, Error};
use frame_support::{
    assert_err, assert_err_ignore_postinfo, assert_noop, assert_ok,
    dispatch::{DispatchError, DispatchErrorWithPostInfo, Dispatchable},
    impl_outer_dispatch, impl_outer_event, impl_outer_origin, parameter_types, storage,
    traits::Filter,
    weights::{Pays, Weight},
};
use sp_core::H256;

fn last_event() -> TestEvent {
    System::events()
        .pop()
        .map(|e| e.event)
        .expect("Event expected")
}

#[test]
fn should_slash_work_test() {
    new_test_ext().execute_with(|| {});
}
