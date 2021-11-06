#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GamesResponse, InstantiateMsg, QueryMsg};
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
    let opponent: Addr = deps.api.addr_validate(&addr)?;

    let empty_check = GAMES.may_load(deps.storage, (&host, &opponent))?;

    if empty_check != None {
        return Err(ContractError::GameInProgress);
    }

    let game_data = Data {
        host_move: first_move,
        opponent_move: GameMove::NoMove,
        result: GameResult::Started,
    };
    GAMES.save(deps.storage, (&host, &opponent), &game_data)?;

    Ok(Response::new().add_attribute("method", "start_game"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryHostGames { addr } => to_binary(&query_host_games(deps, addr)?),
        QueryMsg::QueryAllGames {} => to_binary(&query_all_games(deps)?),
    }
}

pub fn query_host_games(deps: Deps, host_addr: String) -> StdResult<GamesResponse> {
    let host_checked = deps.api.addr_validate(&host_addr)?;

    // get all under one key
    let all = GAMES
        .prefix(&host_checked)
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<_>>()?;
    Ok(GamesResponse { games: all })
    // TODO: I want to do this without having to rely on .unwrap
    // Is it possible to do this without an unwrap??
}

pub fn query_all_games(deps: Deps) -> StdResult<GamesResponse> {
    let all = GAMES
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    Ok(GamesResponse { games: all })
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

    #[test]
    fn start_two_games_same_host_and_opponent() {
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
            addr: String::from("an_opponent"),
            first_move: GameMove::Scissors,
        };
        let res = execute(deps.as_mut(), mock_env(), same_start_info, msg);
        match res {
            Err(ContractError::GameInProgress) => {}
            _ => panic!("Must return ContractError::GameInProgress"),
        }
    }

    #[test]
    fn start_two_games_one_host_different_opponents() {
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
            Err(ContractError::GameInProgress) => panic!("Second game should start"),
            _ => {}
        }
    }

    #[test]
    fn test_query_games() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Start a game
        let start_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            addr: String::from("an_opponent"),
            first_move: GameMove::Scissors,
        };
        let _res = execute(deps.as_mut(), mock_env(), start_info, msg);

        // Start another game with same host
        let same_start_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            addr: "diff_opponent".to_string(),
            first_move: GameMove::Rock,
        };
        let _res = execute(deps.as_mut(), mock_env(), same_start_info, msg);

        // Start another game with different host
        let same_start_info = mock_info("user", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            addr: "diff_opponent".to_string(),
            first_move: GameMove::Rock,
        };
        let _res = execute(deps.as_mut(), mock_env(), same_start_info, msg);

        // Query all games
        let msg = QueryMsg::QueryAllGames {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: GamesResponse = from_binary(&res).unwrap();
        println!("{:?}", value.games);
        assert_eq!(value.games.len(), 3);

        // Query host games
        let msg = QueryMsg::QueryHostGames {
            addr: "creator".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: GamesResponse = from_binary(&res).unwrap();
        assert_eq!(value.games.len(), 2);
        // TODO: Assert that the value of value.games is the same as expected
    }
}
