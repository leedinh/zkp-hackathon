#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_json_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};
use cosmwasm_std::{Coin, CosmosMsg};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::helpers::{validate_balance, validate_sent_funds};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ResultResponse};
use crate::state::{rand_move, Game, GameMove, GameResult, Random, State, GAME, RANDOM, STATE};
use crate::utils::{sha_256, Prng};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:rps";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
    };

    let rand_state = Random {
        prng_seed: sha_256(base64::encode(msg.prng_seed.clone()).as_bytes()).to_vec(),
        entropy: msg.prng_seed.as_bytes().to_vec(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    RANDOM.save(deps.storage, &rand_state)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::StartGame {
            opponent,
            first_move,
        } => try_start_game(deps, info, opponent, first_move),

        ExecuteMsg::Respond { host, second_move } => {
            try_respond_to_game(deps, info, host, second_move)
        }

        ExecuteMsg::BetToken {
            first_move,
            entropy,
        } => try_bet_token(deps, info, env, first_move, entropy),

        ExecuteMsg::Withdraw { coin } => try_withdraw(deps, info, env, coin),
    }
}

pub fn try_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    coin: Coin,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    let contract_addr = env.contract.address.clone();
    let amount = coin.amount.u128();
    let denom = coin.denom.clone();
    validate_balance(&deps, &contract_addr, &denom, amount)?;

    if amount == 0 {
        return Err(ContractError::NoFunds {});
    }

    let res = Response::new()
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: coins(amount.into(), denom.clone()),
        })
        .add_attribute("action", "withdraw")
        .add_attribute("amount", amount.to_string())
        .add_attribute("denom", denom);

    Ok(res)
}

pub fn try_bet_token(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    opp_move: GameMove,
    entropy: String,
) -> Result<Response, ContractError> {
    // validate host address

    let fund = validate_sent_funds(info.funds)?;
    let contract_addr = env.contract.address.clone();

    // Validate contract balance
    validate_balance(&deps, &contract_addr, &fund.denom, fund.amount.u128())?;

    let opponent = info.sender.clone();
    let mut rand_state = RANDOM.load(deps.storage)?;
    let rng = Prng::new_rand_bytes(&rand_state.entropy, (&entropy).as_ref());
    rand_state.entropy = rng.clone();
    RANDOM.save(deps.storage, &rand_state)?;

    let contract_move: GameMove = rand_move(&rng);

    let result = match opp_move {
        GameMove::Rock {} => match contract_move {
            GameMove::Rock {} => GameResult::Tie {},
            GameMove::Paper {} => GameResult::ContractWins {},
            GameMove::Scissors {} => GameResult::PlayerWins {},
        },
        GameMove::Paper {} => match contract_move {
            GameMove::Rock {} => GameResult::PlayerWins {},
            GameMove::Paper {} => GameResult::Tie {},
            GameMove::Scissors {} => GameResult::ContractWins {},
        },
        GameMove::Scissors {} => match contract_move {
            GameMove::Rock {} => GameResult::ContractWins {},
            GameMove::Paper {} => GameResult::PlayerWins {},
            GameMove::Scissors {} => GameResult::Tie {},
        },
    };
    let denom = fund.denom.clone();
    let amount = fund.amount.u128() as u64;

    let message = match result {
        GameResult::PlayerWins {} => Some(BankMsg::Send {
            to_address: opponent.to_string(),
            amount: coins((amount * 2).into(), denom),
        }),
        GameResult::ContractWins {} => Some(BankMsg::Send {
            to_address: opponent.to_string(),
            amount: coins(amount.into(), denom),
        }),
        _ => None,
    };

    if let Some(msg) = message {
        Ok(Response::new()
            .add_message(CosmosMsg::Bank(msg))
            .add_attribute("action", "bet_token")
            .add_attribute("result", result.to_string()))
    } else {
        Ok(Response::new()
            .add_attribute("action", "bet_token")
            .add_attribute("result", result.to_string()))
    }
}

pub fn try_respond_to_game(
    deps: DepsMut,
    info: MessageInfo,
    host: String,
    second_move: GameMove,
) -> Result<Response, ContractError> {
    // validate host address
    let host_addr = deps.api.addr_validate(&host)?;
    let responder_addr = info.sender;

    //load game by passing host addr and opponent addr
    let mut game = GAME.load(deps.storage, (host_addr.clone(), responder_addr.clone()))?;

    if game.opponent != responder_addr {
        return Err(ContractError::UnauthorizedOpponent {});
    } else {
        game.opp_move = Some(second_move);
        game.result = get_game_result(game.clone());
    }

    let game_response = ResultResponse {
        result: game.result.unwrap(),
    };
    GAME.remove(deps.storage, (host_addr, responder_addr));

    Ok(game_response.into())
}

pub fn get_game_result(game: Game) -> Option<GameResult> {
    let host_move = game.host_move;
    let opp_move = game.opp_move?;

    match host_move {
        GameMove::Rock {} => match opp_move {
            GameMove::Rock {} => Some(GameResult::Tie {}),
            GameMove::Paper {} => Some(GameResult::OpponentWins {}),
            GameMove::Scissors {} => Some(GameResult::HostWins {}),
        },
        GameMove::Paper {} => match opp_move {
            GameMove::Rock {} => Some(GameResult::HostWins {}),
            GameMove::Paper {} => Some(GameResult::Tie {}),
            GameMove::Scissors {} => Some(GameResult::OpponentWins {}),
        },
        GameMove::Scissors {} => match opp_move {
            GameMove::Rock {} => Some(GameResult::OpponentWins {}),
            GameMove::Paper {} => Some(GameResult::HostWins {}),
            GameMove::Scissors {} => Some(GameResult::Tie {}),
        },
    }
}

pub fn try_start_game(
    deps: DepsMut,
    info: MessageInfo,
    opponent: String,
    first_move: GameMove,
) -> Result<Response, ContractError> {
    // validate opponent address
    let opponent_addr = deps.api.addr_validate(&opponent)?;

    // try to start game, if game is already started with given host it will throw error, otherwise it will create a new game object and save it under host key
    let start_game = |host: Option<Game>| -> Result<Game, ContractError> {
        match host {
            Some(_) => Err(ContractError::AlreadyStarted {}),
            None => {
                let game = Game {
                    host: info.sender.clone(),
                    opponent: opponent_addr.clone(),
                    host_move: first_move.clone(),
                    opp_move: None,
                    result: None,
                };

                Ok(game)
            }
        }
    };

    GAME.update(
        deps.storage,
        (info.sender.clone(), opponent_addr.clone()),
        start_game,
    )?;

    let res = Response::new()
        .add_attribute("action", "start_game")
        .add_attribute("host", info.sender)
        .add_attribute("opponent", opponent_addr)
        .add_attribute("host_move", first_move.to_string());

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetResult { host, opponent } => {
            to_json_binary(&query_result(deps, host, opponent)?)
        }
    }
}

pub fn query_result(deps: Deps, host: String, opponent: String) -> StdResult<ResultResponse> {
    // validate host address
    let validated_host = deps.api.addr_validate(&host)?;
    let validated_opponent = deps.api.addr_validate(&opponent)?;

    // load game map
    let game = GAME.may_load(deps.storage, (validated_host, validated_opponent))?;

    // get game result if its exist otherwise throw error
    match game {
        Some(game) => match game.result {
            Some(result) => Ok(ResultResponse { result }),
            None => Err(StdError::not_found("The game still has no winner")),
        },
        None => Err(StdError::not_found("Host has not started a game")),
    }
}

#[cfg(test)]
mod tests {
    use crate::state::GameMove;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, Addr};

    #[test]
    fn start_game() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = ExecuteMsg::StartGame {
            opponent: "opponent".to_string(),
            first_move: GameMove::Paper {},
        };

        // try to start game
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // load game map
        let game = GAME
            .load(
                &deps.storage,
                (info.sender.clone(), Addr::unchecked("opponent")),
            )
            .unwrap();

        assert_eq!(info.sender, game.host);
        assert_eq!(GameMove::Paper {}, game.host_move);
        assert_eq!(Addr::unchecked("opponent"), game.opponent);
        assert_eq!(None, game.opp_move);
        assert_eq!(None, game.result);
    }

    #[test]
    fn start_game_with_same_host_diff_opponent() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = ExecuteMsg::StartGame {
            opponent: "opponent".to_string(),
            first_move: GameMove::Paper {},
        };

        // try to start game
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::StartGame {
            opponent: "opponent2".to_string(),
            first_move: GameMove::Paper {},
        };

        // try to start second game
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // load games map
        let game1 = GAME
            .load(
                &deps.storage,
                (info.sender.clone(), Addr::unchecked("opponent")),
            )
            .unwrap();
        let game2 = GAME
            .load(
                &deps.storage,
                (info.sender.clone(), Addr::unchecked("opponent2")),
            )
            .unwrap();

        assert_eq!(info.sender, game1.host);
        assert_eq!(GameMove::Paper {}, game1.host_move);
        assert_eq!(Addr::unchecked("opponent"), game1.opponent);
        assert_eq!(None, game1.opp_move);
        assert_eq!(None, game1.result);

        assert_eq!(info.sender, game2.host);
        assert_eq!(GameMove::Paper {}, game2.host_move);
        assert_eq!(Addr::unchecked("opponent2"), game2.opponent);
        assert_eq!(None, game2.opp_move);
        assert_eq!(None, game2.result);
    }
}
