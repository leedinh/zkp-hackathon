use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("This address already started a game")]
    AlreadyStarted {},

    #[error("Host has not started a game")]
    GameNotFound {},

    #[error("The game still has no winner")]
    GameNotFinished {},

    #[error("You cant respond to this game")]
    UnauthorizedOpponent {},

    #[error("No funds available to withdraw")]
    NoFunds {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
