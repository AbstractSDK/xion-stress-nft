use abstract_std::objects::version_control::VersionControlContract;
use cosmwasm_std::{ensure, Addr, DepsMut, Empty, MessageInfo, Response, StdError};
use cw721_base::{state::TokenInfo, ContractError, Cw721Contract, Extension};
use cw_storage_plus::{Item, Map};

pub const ABSTRACT_VERSION_CONTROL: Item<VersionControlContract> = Item::new("abstract-config");
pub const MINTED: Map<&Addr, bool> = Map::new("abstract-minted");

pub fn abstract_account_mint(
    tract: &Cw721Contract<Extension, Empty, Empty, Empty>,
    mut deps: DepsMut,
    info: MessageInfo,
    // We just use the sender, token_id is never used
    _token_id: String,
    // We just use the sender, owner is never used
    _owner: String,
    token_uri: Option<String>,
    extension: Extension,
) -> Result<Response, ContractError> {
    // We assert the sender is an abstract account. They can only mint 1
    assert_abstract_can_mint(deps.branch(), &info.sender)?;

    let token_id = info.sender.to_string();
    // create the token
    let token = TokenInfo {
        owner: info.sender.clone(),
        approvals: vec![],
        token_uri,
        extension,
    };
    tract
        .tokens
        .update(deps.storage, &token_id, |old| match old {
            Some(_) => Err(ContractError::Claimed {}),
            None => Ok(token),
        })?;

    tract.increment_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("minter", &info.sender)
        .add_attribute("owner", info.sender)
        .add_attribute("token_id", token_id))
}

fn assert_abstract_can_mint(deps: DepsMut, sender: &Addr) -> Result<(), ContractError> {
    let abstract_config = ABSTRACT_VERSION_CONTROL.load(deps.storage)?;

    // We verify the proxy is who they say they are
    abstract_config
        .assert_proxy(sender, &deps.querier)
        .map_err(|e| StdError::generic_err(format!("Sender not a proxy: {e}")))?;

    ensure!(
        MINTED.may_load(deps.storage, sender)?.is_none(),
        StdError::generic_err("Can't mint more than 1 NFT.")
    );

    MINTED.save(deps.storage, sender, &true)?;

    Ok(())
}
