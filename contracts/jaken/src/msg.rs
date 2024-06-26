use cosmwasm_std::{Addr, Coin, Response};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{Game, GameMove, GameResult};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: Addr,
    pub prng_seed: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    StartGame {
        opponent: String,
        first_move: GameMove,
    },
    Respond {
        host: String,
        second_move: GameMove,
    },

    BetToken {
        first_move: GameMove,
        entropy: String,
    },

    Withdraw {
        coin: Coin,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetResult { host: String, opponent: String },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ResultResponse {
    pub result: GameResult,
}

impl From<ResultResponse> for Response {
    fn from(res: ResultResponse) -> Self {
        Response::new()
            .add_attribute("game_status", "finished")
            .add_attribute("Result", res.result.to_string())
    }
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GameResponse {
    pub result: Vec<Game>,
}
