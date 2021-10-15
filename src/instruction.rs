use crate::{error::GauntletError, state::Fees};
use solana_program::program_error::ProgramError;
use std::convert::TryInto;

pub enum DepositType {
    RAYDIUM,
    RAYDIUM_V4,
}
pub enum WithdrawType {
    RAYDIUM,
    RAYDIUM_V4,
}
pub enum SwapType {
    RAYDIUM,
}
pub enum StrategyType {
    RAY,
    RAYDIUM_LP,
}
pub enum GauntletInstruction {
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person initializing the gauntlet
    /// 1. `[writeable]` The account to store gauntlet state
    /// 2. `[]` gauntlet usdc token account
    /// 3. `[]` token program account
    InitGauntlet {},
    /// 0. `[signer]` The account of the person initializing the gauntlet
    /// 1. `[writable]` The account of gauntlet state
    /// 2. `[writable]` The account to store vault state that not initiialized
    /// 3. `[writable]` The account to store vault strategy state that not initiialized
    /// 4. `[]` deposit token account
    /// 5. `[]` withdraw fee token account
    /// 6. `[]` token program account
    /// 7. `[]` farm reward token account
    /// 8. `[]` farm second reward token account // 없으면 skip
    InitVault {
        fees: Fees,
    },

    /// 0. `[signer]` The account of admin
    /// 1. `[writeable]` the account to store gauntlet state
    /// 2. `[writeable]` the account to store strategy state that not initiialized
    /// 3. `[]` strategy token account
    /// 4. `[]` performance fee token account
    InitStrategy {},

    /// 0. `[signer]` The account of vault admin
    /// 1. `[writable]` The account of gauntlet state
    /// 2. `[writable]` The account of vault strategy state
    /// 3. `[writable]` The account of vault state
    /// 4. `[writable or read]` harvest_accounts: accounts used by Radium (deposit, harvest)
    /// 5. `[writable or read]` swap_reward_to_usdc_accounts: accounts used by Radium (swap) (used to swap first reward token)
    /// 6. '[writable or read] [option]` swap_reward_b_to_usdc_accounts: accounts used by Radium (used to swap second reward token)
    /// 7. `[writable or read]` swap_usdc_to_strategy_accounts: accounts used by Radium (used to swap usdc to strategy token)
    /// 8. `[writable]` vault_reward_account: token account of vault reward(ex. RAY) account (token account owned by pda)
    /// 9. `[writable] [option]` vault_reward_b_account: token account of vault second reward(ex. RAY) account (token account owned by pda)
    /// 10. `[writable]` strategy_token_account: token account of strategy(ex. BTC) account (token account owned by pda)
    /// 11. `[writable]` strategy_account: The account to store strategy state
    /// 12. `[]` gauntlet usdc token account
    UpdateVaultStrategy {
        availability: bool,
        needs_usdc_pool: bool,
    },

    /// Deposit
    /// 0. `[signer]` depositor: The account of depositor
    /// 1. `[writable]` depositor_user_account: The account to store user state
    /// 2. `[writable]` depositor_deposit_token_account: The token(LP) account of depositor
    /// 3. `[]` gauntlet_account: The account to store gauntlet state
    /// 4. `[writable]` vault_account: The account to store vault state
    /// 5. `[writable]` vault_deposit_account: token account of vault account (token account owned by pda)
    /// 6. `[writable]` vault_strategy_account: The account to store vault strategy state
    /// 7. `[writable]` vault_reward_account: token account of vault reward(ex. RAY) account (token account owned by pda)
    /// 8. `[writable] [option]` vault_reward_b_account: token account of vault second reward(ex. RAY) account (token account owned by pda)
    /// 9. `[writable]` strategy_account: The account to store strategy state
    /// 10. `[writable]` strategy_token_account: token account of strategy(ex. BTC) account (token account owned by pda)
    /// 11. `[writable]` usdc_token_account: USDC token account (token account owned by pda)
    /// 12. `[writable or read]` harvest_accounts: accounts used by Radium (deposit, harvest)
    /// 13. `[writable or read]` swap_reward_to_usdc_accounts: accounts used by Radium (swap) (used to swap first reward token)
    /// 14. '[writable or read] [option]` swap_reward_b_to_usdc_accounts: accounts used by Radium (used to swap second reward token)
    /// 15. `[writable or read]` swap_usdc_to_strategy_accounts: accounts used by Radium (used to swap usdc to strategy token)
    Deposit {
        amount: u64,
        deposit_type: DepositType,
    },
    /// Harvest
    /// 0. `[]` gauntlet_account: The account to store gauntlet state
    /// 1. `[writable]` vault_account: The account to store vault state
    /// 2. `[writable]` vault_reward_account: token account of vault reward(ex. RAY) account (token account owned by pda)
    /// 3. `[writable] [option]` vault_reward_b_account: token account of vault second reward(ex. RAY) account (token account owned by pda)
    /// 4. `[writable]` strategy_account: The account to store strategy state
    /// 5. `[writable]` strategy_token_account: token account of strategy(ex. BTC) account (token account owned by pda)
    /// 6. `[writable]` usdc_token_account: USDC token account (token account owned by pda)
    /// 7. `[writable or read]` harvest_accounts: accounts used by Radium (harvest)
    /// 8. `[writable or read]` swap_reward_to_usdc_accounts: accounts used by Radium (used to swap first reward token to usdc)
    /// 9. '[writable or read] [option]` swap_reward_b_to_usdc_accounts: accounts used by Radium (used to swap second reward token to usdc)
    /// 10. `[writable or read]` swap_usdc_to_strategy_accounts: accounts used by Radium (used to swap usdc to strategy token)
    Harvest {
        deposit_type: DepositType,
    },

    /// Withdraw
    /// 1. `[writable]` depositor_user_account: The account to store user state
    /// 2. `[writable]` depositor_deposit_token_account: The token(LP) account of depositor
    /// 2. `[writable]` depositor_reward_token_account: The token(ex. BTC) account of depositor
    /// 3. `[]` gauntlet_account: The account to store gauntlet state
    /// 3. `[]` gauntlet_signer_account: pda account owned by gauntlet program
    /// 4. `[writable]` vault_account: The account to store vault state
    /// 5. `[writable]` vault_deposit_account: token account of vault account (token account owned by pda)
    /// 6. `[writable]` vault_strategy_account: The account to store vault strategy state
    /// 7. `[writable]` vault_reward_account: token account of vault reward(ex. RAY) account (token account owned by pda)
    /// 8. `[writable] [option]` vault_reward_b_account: token account of vault second reward(ex. RAY) account (token account owned by pda)
    /// 9. `[writable]` strategy_account: The account to store strategy state
    /// 10. `[writable]` strategy_token_account: token account of strategy(ex. BTC) account (token account owned by pda)
    /// 11. `[writable]` usdc_token_account: USDC token account (token account owned by pda)
    /// 12. `[writable]` withdraw_fee_account: token account for withdraw fee
    /// 13. `[writable]` performance_fee_account: token account for performance fee
    /// 14. `[writable or read]` harvest_accounts: accounts used by Radium ( harvest, withdraw)
    /// 15. `[writable or read]` swap_reward_to_usdc_accounts: accounts used by Radium (swap) (used to swap first reward token)
    /// 16. '[writable or read] [option]` swap_reward_b_to_usdc_accounts: accounts used by Radium (used to swap second reward token)
    /// 17. `[writable or read]` swap_usdc_to_strategy_accounts: accounts used by Radium (used to swap usdc to strategy token)
    Withdraw {
        amount: u64,
        reward_amount: u64,
        withdraw_type: WithdrawType,
    },
    SwapFarmRewardToUsdc {
        swap_type: SwapType,
    },
    SwapUsdcToStrategyToken {
        swap_type: SwapType,
    },
    SwapFarmRewardToStrategyToken {
        swap_type: SwapType,
    },
    CreateUserAccount {},
}

impl GauntletInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input
            .split_first()
            .ok_or(GauntletError::InstructionUnpackError)?;
        Ok(match tag {
            0 => Self::InitGauntlet {},
            1 => {
                let (performance_fee_numerator, _rest) = Self::unpack_u64(rest)?;
                let (performance_fee_denominator, _rest) = Self::unpack_u64(_rest)?;
                let (withdrawal_fee_numerator, _rest) = Self::unpack_u64(_rest)?;
                let (withdrawal_fee_denominator, _rest) = Self::unpack_u64(_rest)?;
                let fees = Fees {
                    performance_fee_numerator,
                    performance_fee_denominator,
                    withdrawal_fee_numerator,
                    withdrawal_fee_denominator,
                };

                Fees::validate(&fees)?;

                Self::InitVault { fees }
            }
            2 => Self::InitStrategy {},
            3 => {
                let (availability, rest) = Self::unpack_bool(rest)?;
                let (needs_usdc_pool, _rest) = Self::unpack_bool(rest)?;
                Self::UpdateVaultStrategy {
                    availability,
                    needs_usdc_pool,
                }
            }
            4 => {
                let (amount, _rest) = Self::unpack_u64(rest)?;
                let (&deposit_type, _rest) = _rest
                    .split_first()
                    .ok_or(GauntletError::InstructionUnpackError)?;
                Self::Deposit {
                    amount,
                    deposit_type: match deposit_type {
                        0 => DepositType::RAYDIUM,
                        1 => DepositType::RAYDIUM_V4,
                        _ => return Err(GauntletError::InstructionUnpackError.into()),
                    },
                }
            }
            5 => {
                let (amount, _rest) = Self::unpack_u64(rest)?;
                let (reward_amount, _rest) = Self::unpack_u64(_rest)?;
                let (&withdraw_type, _rest) = _rest
                    .split_first()
                    .ok_or(GauntletError::InstructionUnpackError)?;
                Self::Withdraw {
                    amount,
                    reward_amount,
                    withdraw_type: match withdraw_type {
                        0 => WithdrawType::RAYDIUM,
                        1 => WithdrawType::RAYDIUM_V4,
                        _ => return Err(GauntletError::InstructionUnpackError.into()),
                    },
                }
            }
            6 => {
                let (&deposit_type, _rest) = rest
                    .split_first()
                    .ok_or(GauntletError::InstructionUnpackError)?;
                Self::Harvest {
                    deposit_type: match deposit_type {
                        0 => DepositType::RAYDIUM,
                        1 => DepositType::RAYDIUM_V4,
                        _ => return Err(GauntletError::InstructionUnpackError.into()),
                    },
                }
            }
            7 => {
                let (&swap_type, _rest) = rest
                    .split_first()
                    .ok_or(GauntletError::InstructionUnpackError)?;
                Self::SwapFarmRewardToUsdc {
                    swap_type: match swap_type {
                        0 => SwapType::RAYDIUM,
                        _ => return Err(GauntletError::InstructionUnpackError.into()),
                    },
                }
            }
            8 => {
                let (&swap_type, _rest) = rest
                    .split_first()
                    .ok_or(GauntletError::InstructionUnpackError)?;
                Self::SwapUsdcToStrategyToken {
                    swap_type: match swap_type {
                        0 => SwapType::RAYDIUM,
                        _ => return Err(GauntletError::InstructionUnpackError.into()),
                    },
                }
            }
            9 => {
                let (&swap_type, _rest) = rest
                    .split_first()
                    .ok_or(GauntletError::InstructionUnpackError)?;
                Self::SwapFarmRewardToStrategyToken {
                    swap_type: match swap_type {
                        0 => SwapType::RAYDIUM,
                        _ => return Err(GauntletError::InstructionUnpackError.into()),
                    },
                }
            }
            10 => Self::CreateUserAccount {},
            _ => return Err(GauntletError::InstructionUnpackError.into()),
        })
    }

    fn unpack_bool(input: &[u8]) -> Result<(bool, &[u8]), ProgramError> {
        if input.is_empty() {
            return Err(GauntletError::InstructionUnpackError.into());
        }

        let (value, _rest) = input
            .split_first()
            .ok_or(GauntletError::InstructionUnpackError)?;

        match value {
            0 => Ok((false, _rest)),
            1 => Ok((true, _rest)),
            _ => Err(GauntletError::InstructionUnpackError.into()),
        }
    }

    fn unpack_u8(input: &[u8]) -> Result<(u8, &[u8]), ProgramError> {
        if input.is_empty() {
            return Err(GauntletError::InstructionUnpackError.into());
        }
        let (bytes, rest) = input.split_at(1);
        let value = bytes
            .get(..1)
            .and_then(|slice| slice.try_into().ok())
            .map(u8::from_le_bytes)
            .ok_or(GauntletError::InstructionUnpackError)?;
        Ok((value, rest))
    }

    fn unpack_u64(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
        if input.len() < 8 {
            return Err(GauntletError::InstructionUnpackError.into());
        }
        let (bytes, rest) = input.split_at(8);
        let value = bytes
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(GauntletError::InstructionUnpackError)?;
        Ok((value, rest))
    }
}
