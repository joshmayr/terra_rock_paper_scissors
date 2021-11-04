#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Data, GameMove, GameResult, GAMES};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:{{project-name}}";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::StartGame { addr, first_move } => try_start_game(deps, info, addr, first_move),
    }
}

pub fn try_start_game(
    deps: DepsMut,
    info: MessageInfo,
    addr: String,
    first_move: GameMove,
) -> Result<Response, ContractError> {
    let host = info.sender;
    let checked: Addr = deps.api.addr_validate(&addr)?;

    let empty_check = GAMES.may_load(deps.storage, &host)?;

    if empty_check != None {
        return Err(ContractError::GameInProgress);
    }

    let game_data = Data {
        opponent: checked,
        host_move: first_move,
        opponent_move: GameMove::NoMove,
        result: GameResult::Started,
    };
    GAMES.save(deps.storage, &host, &game_data)?;

    Ok(Response::new().add_attribute("method", "start_game"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn invalid_address() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let start_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            addr: String::from("not_a_real_address&DFOUSHDOFUGSDOUFGSDOUGDGSGDFO7d9fgas"),
            first_move: GameMove::Rock,
        };
        let res = execute(deps.as_mut(), mock_env(), start_info, msg);
        match res {
            Err(ContractError) => {}
            _ => panic!("Must return ContractError"),
        }
    }

    #[test]
    fn start_multiple_games() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Start first game
        let start_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            addr: String::from("an_opponent"),
            first_move: GameMove::Scissors,
        };
        let _res = execute(deps.as_mut(), mock_env(), start_info, msg);

        let same_start_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            addr: String::from("different_opponent"),
            first_move: GameMove::Scissors,
        };
        let res = execute(deps.as_mut(), mock_env(), same_start_info, msg);
        match res {
            Err(ContractError::GameInProgress) => {}
            _ => panic!("Must return ContractError::GameInProgress"),
        }

        // Start a new game from different address
        let diff_start_info = mock_info("someone_else", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            addr: String::from("an_opponent"),
            first_move: GameMove::Scissors,
        };
        let res = execute(deps.as_mut(), mock_env(), diff_start_info, msg);
        match res {
            Err(ContractError::GameInProgress) => panic!("Should let second game start"),
            _ => {}
        }
    }
}
