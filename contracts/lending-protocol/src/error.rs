use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val:?}")]
    CustomError{val: String},

    #[error("Missing Desposit Hook")]
    MissingDepositHook {},

    #[error("User does not exist")]
    UserDNE {},

    #[error("Insufficient Desposit")]
    InsufficientDeposit {},
    
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
