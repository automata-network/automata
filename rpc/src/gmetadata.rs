#[cfg(all(feature = "automata", feature = "contextfree"))]
compile_error!("Feature 1 and 2 are mutually exclusive and cannot be enabled together");
#[cfg(all(feature = "automata", feature = "finitestate"))]
compile_error!("Feature 1 and 2 are mutually exclusive and cannot be enabled together");
#[cfg(all(feature = "finitestate", feature = "contextfree"))]
compile_error!("Feature 1 and 2 are mutually exclusive and cannot be enabled together");

use automata_primitives::{AccountId, Block, BlockId, Index};
#[cfg(feature = "contextfree")]
use contextfree_runtime::apis::GmetadataApi as GmetadataRuntimeApi;
#[cfg(feature = "finitestate")]
use finitestate_runtime::apis::GmetadataApi as GmetadataRuntimeApi;
use pallet_gmetadata::datastructures::{GmetadataKey, GmetadataQueryResult, HexBytes};

use sc_light::blockchain::BlockchainHeaderBackend as HeaderBackend;
use sp_api::ProvideRuntimeApi;
use sp_runtime::{codec::Decode, traits::Block as BlockT};
use std::sync::Arc;

use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
const RUNTIME_ERROR: i64 = 1;

#[rpc]
pub trait GmetadataServer<BlockHash> {
    //transfer to substrate address
    #[rpc(name = "gmetadata_queryWithIndex")]
    fn query_with_index(
        &self,
        index_key: Vec<GmetadataKey>,
        value_key: GmetadataKey,
        cursor: HexBytes,
        limit: u64,
    ) -> Result<GmetadataQueryResult>;
}

/// An implementation of DAOPortal specific RPC methods.
pub struct GmetadataApi<C> {
    client: Arc<C>,
}

impl<C> GmetadataApi<C> {
    /// Create new `DAOPortal` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        GmetadataApi { client }
    }
}

impl<C> GmetadataServer<<Block as BlockT>::Hash> for GmetadataApi<C>
where
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: GmetadataRuntimeApi<Block>,
{
    fn query_with_index(
        &self,
        index_key: Vec<GmetadataKey>,
        value_key: GmetadataKey,
        cursor: HexBytes,
        limit: u64,
    ) -> Result<GmetadataQueryResult> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        let result = api
            .query_with_index(&at, index_key, value_key, cursor, limit)
            .map_err(|e| Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Runtime unable to get projects list.".into(),
                data: Some(format!("{:?}", e).into()),
            })?;

        Ok(result)
    }
}
