use cosmwasm_std::StdError;
use cw_controllers::{AdminError, HookError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("No Admin Found")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    Hook(#[from] HookError),

    #[error("The host address is blacklisted")]
    HostAddressBlacklisted {},

    #[error("Only One Game can be Played with the Same Opponent At One Time")]
    OnlyOneGameAtATime {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
