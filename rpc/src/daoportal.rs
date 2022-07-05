#[cfg(all(feature = "automata", feature = "contextfree"))]
compile_error!("Feature 1 and 2 are mutually exclusive and cannot be enabled together");
#[cfg(all(feature = "automata", feature = "finitestate"))]
compile_error!("Feature 1 and 2 are mutually exclusive and cannot be enabled together");
#[cfg(all(feature = "finitestate", feature = "contextfree"))]
compile_error!("Feature 1 and 2 are mutually exclusive and cannot be enabled together");

use automata_primitives::{AccountId, Block, BlockId, Index};
// #[cfg(feature = "automata")]
// use automata_runtime::apis::DAOPortalApi as DAOPortalRuntimeApi;

#[cfg(feature = "contextfree")]
use contextfree_runtime::apis::DAOPortalApi as DAOPortalRuntimeApi;
#[cfg(feature = "contextfree")]
use pallet_daoportal_cf::datastructures::{DAOProposal, Project, ProjectId, ProposalId};

#[cfg(feature = "finitestate")]
use finitestate_runtime::apis::DAOPortalApi as DAOPortalRuntimeApi;
#[cfg(feature = "finitestate")]
use pallet_daoportal::datastructures::{DAOProposal, Project, ProjectId, ProposalId};

use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;

use sc_light::blockchain::BlockchainHeaderBackend as HeaderBackend;
use sp_api::ProvideRuntimeApi;
use sp_runtime::{codec::Decode, traits::Block as BlockT};
use std::sync::Arc;

const RUNTIME_ERROR: i64 = 1;

#[rpc]
pub trait DAOPortalServer<BlockHash> {
    //transfer to substrate address
    #[rpc(name = "daoportal_getProjects")]
    fn get_projects(&self) -> Result<Vec<(ProjectId, Project<AccountId>)>>;

    #[rpc(name = "daoportal_getProposals")]
    fn get_proposals(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<(ProposalId, DAOProposal<AccountId>)>>;

    #[rpc(name = "daoportal_getAllProposals")]
    fn get_all_proposals(&self) -> Result<Vec<(ProjectId, ProposalId, DAOProposal<AccountId>)>>;
}

/// An implementation of DAOPortal specific RPC methods.
pub struct DAOPortalApi<C> {
    client: Arc<C>,
}

impl<C> DAOPortalApi<C> {
    /// Create new `DAOPortal` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        DAOPortalApi { client }
    }
}

impl<C> DAOPortalServer<<Block as BlockT>::Hash> for DAOPortalApi<C>
where
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: DAOPortalRuntimeApi<Block>,
{
    /// get projects list
    fn get_projects(&self) -> Result<Vec<(ProjectId, Project<AccountId>)>> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        let projects_list = api.get_projects(&at).map_err(|e| Error {
            code: ErrorCode::ServerError(RUNTIME_ERROR),
            message: "Runtime unable to get projects list.".into(),
            data: Some(format!("{:?}", e).into()),
        })?;

        Ok(projects_list)
    }

    /// get proposals for a project
    fn get_proposals(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<(ProposalId, DAOProposal<AccountId>)>> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        let proposals_list = api.get_proposals(&at, project_id).map_err(|e| Error {
            code: ErrorCode::ServerError(RUNTIME_ERROR),
            message: "Runtime unable to get projects list.".into(),
            data: Some(format!("{:?}", e).into()),
        })?;

        Ok(proposals_list)
    }

    /// get all projects
    fn get_all_proposals(&self) -> Result<Vec<(ProjectId, ProposalId, DAOProposal<AccountId>)>> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        let proposals_list = api.get_all_proposals(&at).map_err(|e| Error {
            code: ErrorCode::ServerError(RUNTIME_ERROR),
            message: "Runtime unable to get projects list.".into(),
            data: Some(format!("{:?}", e).into()),
        })?;

        Ok(proposals_list)
    }
}
