use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Map;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Data {
    pub host_move: GameMove,
    pub opponent_move: GameMove,
    pub result: GameResult,
}

pub const GAMES: Map<(&Addr, &Addr), Data> = Map::new("games");

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub enum GameMove {
    Rock,
    Paper,
    Scissors,
    NoMove,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub enum GameResult {
    HostWins,
    OpponentWins,
    Tie,
    Started,
}
