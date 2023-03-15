use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item, Map};

use cosmwasm_std::{Addr, Uint128, Coin};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    // Burada kontratinizin state i tutulacak
    pub owner:Addr,
    pub price:Coin,
    pub balance:Uint128,
    pub cw20address:Addr,
}

// Burada State i, blockchain e kaydediyoruz. 
// Bu sekilde buradaki datalar blockchain de kalici olarak kaliyor.
pub const STATE: Item<State> = Item::new("state");
pub const BALANCES: Map<&Addr, Uint128> = Map::new("balance");