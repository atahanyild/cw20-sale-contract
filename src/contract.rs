#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg,
    Uint128, WasmMsg, to_binary,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, InfoResponse};
use crate::state::{State, STATE};

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

// Burada versiyonlarimizi yaratiyor ki ileride migrate edersek kontrati bu bilgileri kullanabilelim
const CONTRACT_NAME: &str = "crates.io:template";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Once state i yaratiyoruz
    let state = State {
        owner: info.sender.clone(),
        price: Coin {
            amount: msg.price,
            denom: msg.denom,
        },
        balance: Uint128::zero(),
        cw20address: msg.cw20address,
    };
    // Kontrat versiyonumuzu kaydediyoruz.
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Yarattigimiz state i blockchain e kalici olarak kaydediyoruz.
    STATE.save(deps.storage, &state)?;
    // Fonksiyonumuz donus yapiyor burada, return degerleri olarak methodun ne oldugunu ve instantiate eden kisinin kim oldugunu donduruyoruz.
    // Bu degerler ozellikle front end kisminda onemli olacak.
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
    // Burada match icinde Execute mesajlari ve onlarin dogru fnksiyonlara yonlendirilmeleri olacak.
    match msg {
        ExecuteMsg::Buy { price, denom } => execute::execute_buy(deps, info, price, denom),
        ExecuteMsg::Receive(msg) => execute::execute_receive(deps, msg),
        ExecuteMsg::WithdrawAll {} => execute::execute_withdraw_all(deps, info.sender),
    }
}

pub mod execute {
    use cosmwasm_std::{to_binary, Coin, CosmosMsg};
    use cw20::Cw20ExecuteMsg;

    use super::*;

    pub fn execute_receive(deps: DepsMut, msg: Cw20ReceiveMsg) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.balance += msg.amount;
            Ok(state)
        })?;

        Ok(Response::default())
    }

    pub fn execute_buy(
        deps: DepsMut,
        info: MessageInfo,
        price: Uint128,
        denom: String,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage).unwrap();
        //checking if our denom equals to denom which client wants to buy
        //see later

        //getting our balance from info.funds
        let mut balance = Coin {
            amount: Uint128::new(0),
            denom: state.price.denom.clone(),
        };

        for fund in &info.funds {
            if fund.denom == balance.denom {
                balance = Coin {
                    amount: balance.amount + fund.amount,
                    denom: fund.denom.clone(),
                }
            }
        }
        //checking if our funds are correctly here
        if balance.amount == Uint128::from(0u128) {
            return Err(ContractError::IncorrectFunds {});
        }

        //we are dividing incoming nativecoin to price of cw20token to find how many cw20tokens we are going to send.
        let amount = match balance.amount.checked_div(state.price.amount) {
            Ok(r) => r,
            Err(_) => return Err(ContractError::DivideByZeroError {}),
        };
        //cw20 transfer msg
        let transfer_cw20_msg = Cw20ExecuteMsg::Transfer {
            recipient: info.sender.into(),
            amount,
        };
        let exec_cw20_transfer = WasmMsg::Execute {
            contract_addr: state.cw20address.into(),
            msg: to_binary(&transfer_cw20_msg)?,
            funds: vec![],
        };
        let cw20_transfer_cosmos_msg: CosmosMsg = exec_cw20_transfer.into();

        let transfer_bank_msg = cosmwasm_std::BankMsg::Send {
            to_address: state.owner.into(),
            amount: info.funds,
        };

        let transfer_bank_cosmos_msg: CosmosMsg = transfer_bank_msg.into();

        //update the balance in state
        let updated_balance = match state.balance.checked_sub(amount) {
            Ok(r) => r,
            Err(_) => return Err(ContractError::SubtractionError {}),
        };
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.balance = updated_balance;
            Ok(state)
        })?;

        Ok(Response::new()
            .add_attribute("action", "buy")
            .add_attribute("amount", amount)
            .add_submessages(vec![
                SubMsg::new(cw20_transfer_cosmos_msg),
                SubMsg::new(transfer_bank_cosmos_msg),
            ]))
    }


    pub fn execute_withdraw_all(deps: DepsMut, sender: Addr) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage).unwrap();

        if state.owner != sender {
            return Err(ContractError::Unauthorized {});
        }

        // create transfer cw20 msg
        let transfer_cw20_msg = Cw20ExecuteMsg::Transfer {
            recipient: state.owner.into(),
            amount: state.balance,
        };
        let exec_cw20_transfer = WasmMsg::Execute {
            contract_addr: state.cw20address.into(),
            msg: to_binary(&transfer_cw20_msg)?,
            funds: vec![],
        };
        let cw20_transfer_cosmos_msg: CosmosMsg = exec_cw20_transfer.into();

        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.balance = Uint128::new(0);
            Ok(state)
        })?;

        Ok(Response::new()
            .add_attribute("action", "withdraw_all")
            .add_attribute("amount", state.balance)
            .add_submessages(vec![SubMsg::new(cw20_transfer_cosmos_msg)]))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    // Burada match icinde Query mesajlari ve onlarin dogru fnksiyonlara yonlendirilmeleri olacak.
    match msg {
        QueryMsg::GetInfo {} => to_binary(&query::query_info(deps)?),
    }
}

pub mod query {
    use crate::msg::InfoResponse;

    use super::*;
    pub fn query_info(deps: Deps) -> StdResult<InfoResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(InfoResponse {
            owner: state.owner,
            cw20address: state.cw20address,
            price: state.price,
            balance: state.balance,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
    };
    use cosmwasm_std::{attr, coins, to_binary, StdError, Uint128, from_binary};

    #[test]
    fn init_test() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            price: Uint128::from(7u128),
            denom: "token".to_string(),
            cw20address: Addr::unchecked("cw20addr"),
        };

        let info = mock_info("creator", &coins(2, "token"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetInfo {}).unwrap();
        let value: InfoResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(7u128), value.price.amount);
    }

    #[test]
    fn buy_test() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let price = Uint128::from(7u128);
        let denom = "token".to_string();
        let ins_msg = InstantiateMsg {
            cw20address: Addr::unchecked("cw20addr"),
            price: price,
            denom: denom.clone(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));
        instantiate(deps.as_mut(), mock_env(), info.clone(), ins_msg);

        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            amount: Uint128::from(10u128),
            sender: "asdf".to_string(),
            msg: to_binary("a").unwrap(),
        });
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        //valid transfer
        let info = mock_info("buyer", &coins(21, "token"));
        let msg = ExecuteMsg::Buy {
            price: price,
            denom: denom.clone(),
        };
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 2);

        //overpay
        let msg = ExecuteMsg::Buy {
            denom: denom.clone(),
            price,
        };
        let info = mock_info("buyer", &coins(25, "token"));
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(_res.attributes[1], &attr("amount", Uint128::from(3u128)));
    }

    #[test]
    fn withdraw_cw20_token() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {
            cw20address: Addr::unchecked("asdf"),
            price: Uint128::from(7u128),
            denom: "token".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            amount: Uint128::from(10u128),
            sender: "asdf".to_string(),
            msg: to_binary("a").unwrap(),
        });
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // check balance
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetInfo {}).unwrap();
        let value: InfoResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(10u128), value.balance);

        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::WithdrawAll {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // check balance
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetInfo {}).unwrap();
        let value: InfoResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::new(0), value.balance);
    }

    #[test]
    fn withdraw_cw20_token_only_creator() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {
            cw20address: Addr::unchecked("asdf"),
            price: Uint128::from(7u128),
            denom: "token".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("imposter", &coins(2, "token"));

        let msg = ExecuteMsg::WithdrawAll {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(_res.is_err());
    }

}
