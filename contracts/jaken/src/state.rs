use std::fmt::Display;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]

pub struct State {
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Game {
    pub host: Addr,
    pub opponent: Addr,
    pub host_move: GameMove,
    pub opp_move: Option<GameMove>,
    pub result: Option<GameResult>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GameMove {
    Rock {},
    Paper {},
    Scissors {},
}

impl Display for GameMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameMove::Rock {} => write!(f, "Rock"),
            GameMove::Paper {} => write!(f, "Paper"),
            GameMove::Scissors {} => write!(f, "Scissors"),
        }
    }
}

pub fn rand_move(rng: &[u8]) -> GameMove {
    let num: u8 = rng[0] % 3 + 1;
    match num {
        1 => GameMove::Rock {},
        2 => GameMove::Paper {},
        3 => GameMove::Scissors {},
        _ => panic!("Invalid random number"),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GameResult {
    HostWins {},
    OpponentWins {},
    Tie {},
    ContractWins(),
    PlayerWins(),
}

impl Display for GameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameResult::HostWins {} => write!(f, "Host Wins!"),
            GameResult::OpponentWins {} => write!(f, "Opponent Wins!"),
            GameResult::Tie {} => write!(f, "Game is Tie !"),
            GameResult::ContractWins() => write!(f, "Contract Wins!"),
            GameResult::PlayerWins() => write!(f, "Player Wins!"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Random {
    pub prng_seed: Vec<u8>,
    pub entropy: Vec<u8>,
}

pub const STATE: Item<State> = Item::new("state");
pub const GAME: Map<(Addr, Addr), Game> = Map::new("game");
pub const RANDOM: Item<Random> = Item::new("random");
