// SPDX-License-Identifier: Apache-2.0

use automata_primitives::{AccountId, Hash};
use codec::{Decode, Encode};
use pallet_fulfillment::Geode;
use sp_core::{H160, U256};
use sp_std::vec::Vec;

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
pub struct EthProposal {
    pub data: Vec<u8>,
}

sp_api::decl_runtime_apis! {
    pub trait FulfillmentApi {
        fn attestor_list() -> Vec<(Vec<u8>, Vec<u8>)>;
        fn registered_geodes() -> Vec<Geode<AccountId, Hash>>;
    }

    pub trait ExpandApi {
        /// Returns the balances of all accounts at a height
        fn get_token_balances(token: H160, at: Option<u32>) -> Vec<(H160, U256)>;
        /// Returns all the current proposals. The proposals belong to the workspace,
        /// so you need to traverse the workspace before fetching the proposals.
        fn get_proposals(contract: H160, space: U256) -> Vec<EthProposal>;
    }
}
