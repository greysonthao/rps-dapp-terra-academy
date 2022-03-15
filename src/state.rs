use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_controllers::{Admin, Hooks};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");
pub const GAME: Map<(&Addr, &Addr), Game> = Map::new("game");
pub const ADMIN: Admin = Admin::new("admin");
pub const HOOKS: Hooks = Hooks::new("cw4-hooks");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Game {
    pub host: Addr,
    pub opponent: Addr,
    pub host_move: GameMove,
    pub opp_move: Option<GameMove>,
    pub result: Option<GameResult>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GameMove {
    Rock,
    Paper,
    Scissors,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GameResult {
    HostWins,
    OpponentWins,
    Tie,
}
