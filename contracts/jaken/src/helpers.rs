use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_json_binary, Addr, Coin, CosmosMsg, DepsMut, StdError, StdResult, WasmMsg};

use crate::msg::ExecuteMsg;

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }
}

pub fn validate_sent_funds(funds: Vec<Coin>) -> Result<Coin, StdError> {
    if funds.len() != 1 {
        return Err(StdError::generic_err(format!(
            "multiple coin sent({:?})",
            funds
        )));
    }

    let fund = &funds[0];
    if fund.amount.is_zero() {
        return Err(StdError::generic_err(format!(
            "sent fund is zero amount({:?})",
            fund
        )));
    }
    Ok(fund.clone())
}

pub fn validate_balance(
    deps: &DepsMut,
    address: &Addr,
    denom: &str,
    amount: u128,
) -> Result<bool, StdError> {
    let balance = deps.querier.query_balance(address, denom)?;
    if balance.amount.u128() < amount as u128 {
        return Err(StdError::generic_err(format!(
            "insufficient balance in address({})",
            address
        )));
    }
    Ok(true)
}
