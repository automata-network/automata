// SPDX-License-Identifier: Apache-2.0

use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn is_work() {
    new_test_ext().execute_with(|| {});
}
