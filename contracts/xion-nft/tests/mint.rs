use abstract_client::{AbstractClient, Account};
use abstract_std::ibc_host::HostAction;
use abstract_std::manager::ExecuteMsgFns;
use abstract_std::{ibc_client, manager, proxy, PROXY};
use cosmwasm_std::{to_json_binary, wasm_execute};
use cw721_base::InstantiateMsg as Cw721InstantiateMsg;
use cw_orch::contract::interface_traits::{CwOrchInstantiate, CwOrchUpload};
use cw_orch::mock::cw_multi_test::AppResponse;
use cw_orch::prelude::*;
use cw_orch_interchain::prelude::*;
use xion_nft::{interface::AbstractNFT, msg::InstantiateMsg};
use xion_nft::{NftExecuteMsg, NftQueryMsg};

pub fn mint_fn(
    account: &Account<MockBech32>,
    nft: &AbstractNFT<MockBech32>,
) -> cw_orch::anyhow::Result<AppResponse> {
    account
        .as_ref()
        .manager
        .execute_on_module(
            PROXY,
            proxy::ExecuteMsg::IbcAction {
                msg: ibc_client::ExecuteMsg::RemoteAction {
                    host_chain: "dst".to_string(),
                    action: HostAction::Dispatch {
                        manager_msgs: vec![manager::ExecuteMsg::ExecOnModule {
                            module_id: PROXY.to_string(),
                            exec_msg: to_json_binary(&proxy::ExecuteMsg::ModuleAction {
                                msgs: vec![wasm_execute(
                                    nft.address()?,
                                    &NftExecuteMsg::Mint {
                                        token_id: "disregarded".to_string(),
                                        owner: "disregarded".to_string(),
                                        token_uri: None,
                                        extension: None,
                                    },
                                    vec![],
                                )?
                                .into()],
                            })?,
                        }],
                    },
                },
            },
        )
        .map_err(Into::into)
}

#[test]
pub fn mint() -> cw_orch::anyhow::Result<()> {
    let interchain = MockBech32InterchainEnv::new(vec![("src-1", "src"), ("dst-1", "dst")]);
    let src = interchain.chain("src-1")?;
    let dst = interchain.chain("dst-1")?;
    let src_abstract = AbstractClient::builder(src.clone()).build()?;
    let dst_abstract = AbstractClient::builder(dst.clone()).build()?;

    crate::setup::ibc_connect_abstract(&interchain, "src-1", "dst-1")?;

    // We upload the contract on the remote chain
    let nft = AbstractNFT::new(dst.clone());
    nft.upload()?;
    nft.instantiate(
        &InstantiateMsg {
            cw721_msg: Cw721InstantiateMsg {
                name: "Xion testnet NFT".to_string(),
                symbol: "XTN".to_string(),
                minter: None,
                withdraw_address: None,
            },
            abstract_vc: dst_abstract.version_control().address()?.to_string(),
        },
        None,
        None,
    )?;

    let src_account = src_abstract.account_builder().build()?;

    src_account.as_ref().manager.update_settings(Some(true))?;

    // Try to mint from the src chain
    let ibc_response = mint_fn(&src_account, &nft)?;
    let _ = interchain.check_ibc("src-1", ibc_response)?;

    let num_tokens: cw721::NumTokensResponse = nft.query(&NftQueryMsg::NumTokens {})?;
    assert_eq!(num_tokens.count, 1);

    // Try to mint a second time an error
    let ibc_response = mint_fn(&src_account, &nft)?;
    let err = interchain
        .wait_ibc("src-1", ibc_response)?
        .into_result()
        .unwrap_err();

    println!("err : {:?}", err);
    assert!(err.to_string().contains("Can't mint more than 1"));

    let num_tokens: cw721::NumTokensResponse = nft.query(&NftQueryMsg::NumTokens {})?;
    assert_eq!(num_tokens.count, 1);

    Ok(())
}

pub mod setup {

    use abstract_interface::connection::abstract_ibc_connection_with;
    use abstract_interface::Abstract;
    use cw_orch::anyhow::Result as AnyResult;
    use cw_orch::prelude::*;
    use cw_orch_interchain::prelude::*;
    use cw_orch_polytone::Polytone;
    use polytone::handshake::POLYTONE_VERSION;

    pub fn ibc_connect_abstract<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>>(
        interchain: &IBC,
        origin_chain_id: &str,
        remote_chain_id: &str,
    ) -> AnyResult<(Abstract<Chain>, Abstract<Chain>)> {
        let origin_chain = interchain.chain(origin_chain_id).unwrap();
        let remote_chain = interchain.chain(remote_chain_id).unwrap();

        // Deploying abstract and the IBC abstract logic
        let abstr_origin = Abstract::load_from(origin_chain.clone())?;
        let abstr_remote = Abstract::load_from(remote_chain.clone())?;

        // Deploying polytone on both chains
        Polytone::deploy_on(origin_chain.clone(), None)?;
        Polytone::deploy_on(remote_chain.clone(), None)?;

        ibc_connect_polytone_and_abstract(interchain, origin_chain_id, remote_chain_id)?;

        Ok((abstr_origin, abstr_remote))
    }

    pub fn ibc_abstract_setup<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>>(
        interchain: &IBC,
        origin_chain_id: &str,
        remote_chain_id: &str,
    ) -> AnyResult<(Abstract<Chain>, Abstract<Chain>)> {
        let origin_chain = interchain.chain(origin_chain_id).unwrap();
        let remote_chain = interchain.chain(remote_chain_id).unwrap();

        // Deploying abstract and the IBC abstract logic
        let abstr_origin =
            Abstract::deploy_on(origin_chain.clone(), origin_chain.sender().to_string())?;
        let abstr_remote =
            Abstract::deploy_on(remote_chain.clone(), remote_chain.sender().to_string())?;

        // Deploying polytone on both chains
        Polytone::deploy_on(origin_chain.clone(), None)?;
        Polytone::deploy_on(remote_chain.clone(), None)?;

        ibc_connect_polytone_and_abstract(interchain, origin_chain_id, remote_chain_id)?;

        Ok((abstr_origin, abstr_remote))
    }

    pub fn ibc_connect_polytone_and_abstract<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>>(
        interchain: &IBC,
        origin_chain_id: &str,
        remote_chain_id: &str,
    ) -> AnyResult<()> {
        let origin_chain = interchain.chain(origin_chain_id).unwrap();
        let remote_chain = interchain.chain(remote_chain_id).unwrap();

        let abstr_origin = Abstract::load_from(origin_chain.clone())?;
        let abstr_remote = Abstract::load_from(remote_chain.clone())?;

        let origin_polytone = Polytone::load_from(origin_chain.clone())?;
        let remote_polytone = Polytone::load_from(remote_chain.clone())?;

        // Creating a connection between 2 polytone deployments
        interchain.create_contract_channel(
            &origin_polytone.note,
            &remote_polytone.voice,
            POLYTONE_VERSION,
            None, // Unordered channel
        )?;
        // Create the connection between client and host
        abstract_ibc_connection_with(&abstr_origin, interchain, &abstr_remote, &origin_polytone)?;
        Ok(())
    }
}
