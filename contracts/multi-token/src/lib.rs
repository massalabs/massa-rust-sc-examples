//! An MRC1155 multi-token with per-token metadata, role-gated minting
//! (`minter` role managed by the owner through role-based access control) and
//! burning.
//!
//! Deploy with `constructor(uri: string)`. The deployer becomes the owner and
//! the first minter.
#![cfg_attr(target_arch = "wasm32", no_std)]

use massa_sc_sdk::prelude::*;
use massa_sc_standards::mrc1155::{self, metadata, mintable::MINTER_ROLE};
use massa_sc_standards::{
    access_control, export_access_control, export_mrc1155_burnable, export_mrc1155_metadata,
    export_mrc1155_mintable, export_ownable, ownership,
};

#[export]
pub fn constructor(mut args: Args) {
    let uri: String = args.must_next("uri");
    mrc1155::constructor(&uri);
    access_control::grant_role(MINTER_ROLE, &context::caller());
}

// version, uri (per-token, with base URI fallback), balanceOf, balanceOfBatch,
// setApprovalForAll, isApprovedForAll, safeTransferFrom, safeBatchTransferFrom
export_mrc1155_metadata!();
export_mrc1155_mintable!();
export_mrc1155_burnable!();
export_access_control!();
export_ownable!();

/// `set_token_uri(id: u256, uri: string)`: owner only.
#[export]
pub fn set_token_uri(mut args: Args) {
    ownership::only_owner();
    let id: U256 = args.must_next("id");
    let uri: String = args.must_next("uri");
    metadata::set_uri(id, &uri);
}

/// `set_base_uri(uri: string)`: owner only.
#[export]
pub fn set_base_uri(mut args: Args) {
    ownership::only_owner();
    let uri: String = args.must_next("uri");
    metadata::set_base_uri(&uri);
}

#[cfg(test)]
mod tests {
    use super::*;
    use massa_sc_sdk::mock;

    #[test]
    fn mint_with_role_and_metadata() {
        mock::reset();
        mock::set_deploy_context(None);
        constructor(Args::new().add("ipfs://collection/{id}.json"));
        mock::set_admin_context(false);
        let owner = context::caller();
        assert!(access_control::has_role(MINTER_ROLE, &owner));
        mrc1155::mintable::entrypoints::mint(
            Args::new().add(&owner).add(U256::from(7u64)).add(U256::from(100u64)).add(Vec::<u8>::new()),
        );
        assert_eq!(mrc1155::balance_of(&owner, U256::from(7u64)), U256::from(100u64));
        set_base_uri(Args::new().add("https://meta/"));
        set_token_uri(Args::new().add(U256::from(7u64)).add("7.json"));
        assert_eq!(metadata::uri(U256::from(7u64)), "https://meta/7.json");
        assert_eq!(metadata::uri(U256::from(8u64)), "ipfs://collection/{id}.json");
    }
}
