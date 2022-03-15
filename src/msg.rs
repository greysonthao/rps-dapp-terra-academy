use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{Game, GameMove};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    StartGame { opponent: Addr, host_move: GameMove },
    UpdateAdmin { admin: Addr },
    AddToBlacklist { address: Addr },
    RemoveFromBlacklist { address: Addr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetGamesByOpponent { opponent: Addr },
    GetGamesByHost {},
    GetOwner {},
    GetGame { host: Addr, opponent: Addr },
    GetAdmin {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GamesListResponse {
    pub games: Vec<Game>,
}
