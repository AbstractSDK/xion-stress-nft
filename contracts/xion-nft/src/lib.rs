#[cfg(not(target_arch = "wasm32"))]
pub mod interface;

pub mod mint;
pub mod msg;

use abstract_std::objects::version_control::VersionControlContract;
use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
use cw721_base::{
    ContractError, Cw721Contract, ExecuteMsg, Extension, QueryMsg, CONTRACT_NAME, CONTRACT_VERSION,
};
use mint::{abstract_account_mint, ABSTRACT_CONFIG};
use msg::InstantiateMsg;

pub type NftExecuteMsg = ExecuteMsg<Extension, Empty>;
pub type NftQueryMsg = QueryMsg<Empty>;

// This makes a conscious choice on the various generics used by the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ABSTRACT_CONFIG.save(
        deps.storage,
        &VersionControlContract {
            address: deps.api.addr_validate(&msg.abstract_vc)?,
        },
    )?;

    let tract = Cw721Contract::<Extension, Empty, Empty, Empty>::default();
    tract.instantiate(deps, env, info, msg.cw721_msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: NftExecuteMsg,
) -> Result<Response, ContractError> {
    let tract = Cw721Contract::<Extension, Empty, Empty, Empty>::default();

    // We change the mint function, everyone can mint
    match msg {
        ExecuteMsg::Mint {
            token_id,
            owner,
            token_uri,
            extension,
        } => abstract_account_mint(&tract, deps, info, token_id, owner, token_uri, extension),
        _ => tract.execute(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: NftQueryMsg) -> StdResult<Binary> {
    let tract = Cw721Contract::<Extension, Empty, Empty, Empty>::default();
    tract.query(deps, env, msg)
}
