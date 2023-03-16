use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Binary, Coin};
use cw20::{Cw20ReceiveMsg};

#[cw_serde]
pub struct InstantiateMsg {
    // Buradaki mesajiniz ile instantiate edilecek kontrat.
    pub price:Uint128,
    pub denom:String,
    pub cw20address:Addr,
    
}
#[cw_serde]
pub enum ExecuteMsg{
    Buy {  },
    Receive(Cw20ReceiveMsg),
    WithdrawAll {},
    SetPrice { denom: String, price: Uint128 },
}
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(InfoResponse)]
    GetInfo {},
}

#[cw_serde]
pub struct InfoResponse {
    pub owner: Addr,
    pub cw20address: Addr,
    pub price: Coin,
    pub balance: Uint128,
}