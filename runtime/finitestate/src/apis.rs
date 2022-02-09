use automata_primitives::{AccountId, Hash};
use pallet_daoportal::datastructures::{ProjectId, Project, ProposalId, Proposal};
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

        fn get_proposals(project_id: ProjectId) -> Vec<(ProposalId, Proposal<AccountId>)>;

        fn get_all_proposals() -> Vec<(ProjectId, ProposalId, Proposal<AccountId>)>;
    }
}
