use cw_orch::prelude::*;
use cw_orch::{
    contract::{artifacts_dir_from_workspace, interface_traits::Uploadable},
    interface,
};

use crate::msg::InstantiateMsg;
use crate::{NftExecuteMsg, NftQueryMsg};

#[interface(InstantiateMsg, NftExecuteMsg, NftQueryMsg, Empty, id = "remote-nft")]
pub struct AbstractNFT;

impl<Chain> Uploadable for AbstractNFT<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("xion-nft")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(crate::execute, crate::instantiate, crate::query)
                .with_migrate(crate::migrate),
        )
    }
}
