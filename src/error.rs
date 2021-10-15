use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum GauntletError {
    /// Invalid instruction data passed in.
    #[error("Failed to unpack instruction data")]
    InstructionUnpackError,
    /// The provided fee does not match the program owner's constraints
    #[error("The provided fee does not match the program owner's constraints")]
    InvalidFee,
    /// Signer is not Admin
    #[error("Wrong Account")]
    InvalidAccount,
    /// Signer is not Admin
    #[error("Signer is not admin")]
    NotAdmin,
    /// Add exist reward token strategy error
    #[error("Add exist reward token strategy error")]
    ExistRewardTokenStrategy,
    /// not registered strategy
    #[error("Not Registered Strategy")]
    NotRegisteredStrategy,
    /// Vault has max strategy
    #[error("Vault has max strategy")]
    VaultHasMaxStrategy,
    /// Wrong Deposit Token
    #[error("Wrong Deposit Token")]
    WrongDepositToken,
    /// withdraw amount error
    #[error("Can not withdraw more than deposit amount")]
    WithdrawAmountError,
    /// duplicate farm reward token
    #[error("Duplicate Farm Reward Token")]
    DuplicateFarmRewardToken,
    /// wrong user state account
    #[error("Wrong User Account")]
    WrongUserAccount,
    /// Withdraw type error
    #[error("Withdraw type error")]
    WithdrawTypeError,
    /// Deposit type error
    #[error("Deposit type error")]
    DepositTypeError,
    /// Strategy Id error
    #[error("Strategy id is bigger than strategy length")]
    StrategyIdSizeError,
    #[error("Wrong reward token account")]
    RewardTokenAccountError,
    #[error("Invalid user Status")]
    UserStatusError,
    #[error("Timeout Error")]
    TimeoutError,
    #[error("Wrong Vault state account")]
    WrongVaultStateAccount,
    #[error("Wrong Strategy state account")]
    WrongStrategyStateAccount,
    #[error("Wrong Vault Strategy account")]
    WrongVaultStrategyStateAccount,
    #[error("Wrong token account")]
    WrongTokenAccount,
    #[error("Wrong fee account")]
    WrongFeeAccount,
    #[error("Invalid status strategy")]
    InvalidStatusStrategy,
    #[error("Wrong Withdraw amount")]
    InvalidWithdrawAmount,
    #[error("Wrong program id")]
    InvalidProgramId,
}

impl From<GauntletError> for ProgramError {
    fn from(e: GauntletError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
