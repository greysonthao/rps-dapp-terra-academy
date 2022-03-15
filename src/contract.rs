#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdError, StdResult,
};
use cw0::maybe_addr;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GamesListResponse, InstantiateMsg, QueryMsg};
use crate::state::{Game, GameMove, State, ADMIN, GAME, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:rps-dapp";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin_address = maybe_addr(deps.api, Some(info.sender.to_string()))?;

    ADMIN.set(deps.branch(), admin_address)?;

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", &info.sender)
        .add_attribute("admin", &info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::StartGame {
            opponent,
            host_move,
        } => try_start_game(deps, info, opponent, host_move),
        ExecuteMsg::UpdateAdmin { admin } =>
        //TO DO
        {
            try_admin_update(deps, info, admin)
        }
    }
}

pub fn try_admin_update(
    deps: DepsMut,
    info: MessageInfo,
    admin: Addr,
) -> Result<Response, ContractError> {
    let val_addr = maybe_addr(deps.api, Some(admin.to_string()))?;

    return Ok(ADMIN.execute_update_admin(deps, info, val_addr)?);
}

pub fn try_start_game(
    deps: DepsMut,
    info: MessageInfo,
    opponent: Addr,
    host_move: GameMove,
) -> Result<Response, ContractError> {
    let val_addr = deps.api.addr_validate(opponent.as_str())?;

    let game = GAME.may_load(deps.storage, (&info.sender, &val_addr))?;

    match game {
        Some(_) => return Err(ContractError::OnlyOneGameAtATime {}),
        None => {
            let game_info = Game {
                host: info.sender.clone(),
                opponent: val_addr.clone(),
                host_move: host_move,
                opp_move: None,
                result: None,
            };

            GAME.save(deps.storage, (&info.sender, &val_addr), &game_info)?;
        }
    }

    Ok(Response::new().add_attribute("method", "try_start_game"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOwner {} => to_binary(&query_owner(deps)?),
        QueryMsg::GetGamesByHost {} => to_binary(&query_game_by_host(deps)?),
        QueryMsg::GetGamesByOpponent { opponent } => to_binary(&query_game_by_opp(deps, opponent)?),
        QueryMsg::GetGame { host, opponent } => to_binary(&query_game(deps, host, opponent)?),
        QueryMsg::GetAdmin {} => to_binary(&query_admin(deps)?),
    }
}

fn query_owner(deps: Deps) -> StdResult<Addr> {
    let state = STATE.load(deps.storage)?;
    Ok(Addr::from(state.owner))
}

fn query_game(deps: Deps, host: Addr, opponent: Addr) -> StdResult<Game> {
    let val_host_addr = deps.api.addr_validate(&host.as_str())?;
    let val_opp_addr = deps.api.addr_validate(&opponent.as_str())?;

    let game = GAME.may_load(deps.storage, (&val_host_addr, &val_opp_addr))?;

    match game {
        Some(g) => Ok(Game {
            host: g.host,
            opponent: g.opponent,
            host_move: g.host_move,
            opp_move: g.opp_move,
            result: g.result,
        }),
        None => Err(StdError::generic_err("Game not found")),
    }
}

fn query_game_by_host(deps: Deps) -> StdResult<GamesListResponse> {
    let state = STATE.load(deps.storage)?;

    let validated_addr = deps.api.addr_validate(&state.owner.as_str())?;

    let mut games_found: Vec<Game> = vec![];

    let games_queried: StdResult<Vec<_>> = GAME
        .prefix(&validated_addr)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    for games_queried in &games_queried? {
        games_found.push(games_queried.1.clone());
    }

    Ok(GamesListResponse { games: games_found })
}

fn query_game_by_opp(deps: Deps, opponent: Addr) -> StdResult<GamesListResponse> {
    let validated_addr = deps.api.addr_validate(&opponent.as_str())?;

    let mut games_found: Vec<Game> = vec![];

    let games_queried: StdResult<Vec<_>> = GAME
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    for games_queried in &games_queried? {
        if validated_addr == games_queried.1.opponent {
            games_found.push(games_queried.1.clone());
        }
    }

    Ok(GamesListResponse { games: games_found })
}

fn query_admin(deps: Deps) -> StdResult<Option<Addr>> {
    Ok(ADMIN.get(deps)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query owner
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();
        let value: Addr = from_binary(&res).unwrap();
        assert_eq!(Addr::unchecked("creator"), value);
    }

    #[test]
    fn query_games_by_host() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // execute start game w/ 1st opponent and host move
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("first_player"),
            host_move: GameMove::Rock,
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // execute start game w/ 2nd opponent and host move
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("second_player"),
            host_move: GameMove::Paper,
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // execute start game w/ 3rd opponent and host move
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("third_player"),
            host_move: GameMove::Scissors,
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query games by host address
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGamesByHost {}).unwrap();

        let value: GamesListResponse = from_binary(&res).unwrap();

        assert_eq!(3, value.games.len());

        assert_eq!(Addr::unchecked("creator"), value.games[0].host);
        assert_eq!(Addr::unchecked("first_player"), value.games[0].opponent);
        assert_eq!(GameMove::Rock, value.games[0].host_move);
        assert_eq!(None, value.games[0].opp_move);
        assert_eq!(None, value.games[0].result);

        assert_eq!(Addr::unchecked("creator"), value.games[1].host);
        assert_eq!(Addr::unchecked("second_player"), value.games[1].opponent);
        assert_eq!(GameMove::Paper, value.games[1].host_move);
        assert_eq!(None, value.games[1].opp_move);
        assert_eq!(None, value.games[1].result);
    }

    #[test]
    fn query_games_by_opp() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // execute start game w/ opponent and host move
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("other_player"),
            host_move: GameMove::Rock,
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query games by opponent address
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetGamesByOpponent {
                opponent: Addr::unchecked("other_player"),
            },
        )
        .unwrap();

        let value: GamesListResponse = from_binary(&res).unwrap();

        assert_eq!(Addr::unchecked("creator"), value.games[0].host);
        assert_eq!(Addr::unchecked("other_player"), value.games[0].opponent);
        assert_eq!(GameMove::Rock, value.games[0].host_move);
        assert_eq!(None, value.games[0].opp_move);
        assert_eq!(None, value.games[0].result);
    }

    #[test]
    fn query_game_by_opp_and_host() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // execute start game w/ opponent and host move
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("other_player"),
            host_move: GameMove::Rock,
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query games by host and opponent addresses - fail
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetGame {
                host: Addr::unchecked("creator"),
                opponent: Addr::unchecked("not_a_real_player"),
            },
        );

        match res {
            Err(_std_error) => {}
            _ => panic!("Must return Game not found error"),
        }

        // query games by host and opponent addresses - success
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetGame {
                host: Addr::unchecked("creator"),
                opponent: Addr::unchecked("other_player"),
            },
        )
        .unwrap();

        let value: Game = from_binary(&res).unwrap();

        assert_eq!(Addr::unchecked("creator"), value.host);
        assert_eq!(Addr::unchecked("other_player"), value.opponent);
        assert_eq!(GameMove::Rock, value.host_move);
        assert_eq!(None, value.opp_move);
        assert_eq!(None, value.result);
    }

    #[test]
    fn get_admin() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info("creator_man", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query admin success
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAdmin {}).unwrap();
        let value: Addr = from_binary(&res).unwrap();
        assert_eq!(Addr::unchecked("creator_man"), value);
    }

    /* #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    } */
}
