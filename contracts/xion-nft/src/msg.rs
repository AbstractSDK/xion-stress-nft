use cosmwasm_schema::cw_serde;
pub use cw721_base::InstantiateMsg as Cw721InstantiateMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub cw721_msg: Cw721InstantiateMsg,
    pub abstract_vc: String,
}

#[cw_serde]
pub struct MigrateMsg {}
