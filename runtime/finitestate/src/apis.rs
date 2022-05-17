use automata_primitives::{AccountId, Hash};
use pallet_daoportal::datastructures::{DAOProposal, Project, ProjectId, ProposalId};
use pallet_gmetadata::datastructures::{GmetadataKey, GmetadataQueryResult, HexBytes};
use sp_std::vec::Vec;

// use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    pub trait TransferApi {
        fn submit_unsigned_transaction(
            message: [u8; 72],
            signature_raw_bytes: [u8; 65]
        ) -> Result<(), ()>;
    }

    pub trait DAOPortalApi {
        fn get_projects() -> Vec<(ProjectId, Project<AccountId>)>;

        fn get_proposals(project_id: ProjectId) -> Vec<(ProposalId, DAOProposal<AccountId>)>;

        fn get_all_proposals() -> Vec<(ProjectId, ProposalId, DAOProposal<AccountId>)>;
    }

    pub trait GmetadataApi {
        fn query_with_index(
            index_key: GmetadataKey, 
            value_key: GmetadataKey, 
            cursor: HexBytes, 
            limit: u64
        ) -> GmetadataQueryResult;
    }
}
