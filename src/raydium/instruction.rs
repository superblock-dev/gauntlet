use solana_program::program_error::ProgramError;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use std::mem::size_of;
pub enum RaydiumInstruction {
    Deposit { amount: u64 },
    DepositV4 { amount: u64 },
    Harvest {},
    HarvestV4 {},
    Withdraw { amount: u64 },
    WithdrawV4 { amount: u64 },
    Swap { amount_in: u64, amount_out: u64 },
}

impl RaydiumInstruction {
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            RaydiumInstruction::Deposit { amount } => {
                buf.push(1);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            RaydiumInstruction::DepositV4 { amount } => {
                buf.push(1);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            RaydiumInstruction::Harvest {} => {
                let amount: u64 = 0;
                buf.push(1);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            RaydiumInstruction::HarvestV4 {} => {
                let amount: u64 = 0;
                buf.push(1);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            RaydiumInstruction::Withdraw { amount } => {
                buf.push(2);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            RaydiumInstruction::WithdrawV4 { amount } => {
                buf.push(2);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            RaydiumInstruction::Swap {
                amount_in,
                amount_out,
            } => {
                buf.push(9);
                buf.extend_from_slice(&amount_in.to_le_bytes());
                buf.extend_from_slice(&amount_out.to_le_bytes());
            }
        };
        buf
    }
    pub fn deposit(
        stake_program_id: &Pubkey,
        pool_id: &Pubkey,
        pool_authority: &Pubkey,
        user_info_account: &Pubkey,
        user_owner: &Pubkey,
        user_lp_token_account: &Pubkey,
        pool_lp_token_account: &Pubkey,
        user_reward_token_account: &Pubkey,
        pool_reward_token_account: &Pubkey,
        clock_account: &Pubkey,
        spl_token_program: &Pubkey,
        amount: u64,
    ) -> Result<Instruction, ProgramError> {
        let data = RaydiumInstruction::Deposit { amount }.pack();
        let mut accounts = Vec::with_capacity(10);
        accounts.push(AccountMeta::new(*pool_id, false));
        accounts.push(AccountMeta::new_readonly(*pool_authority, false));
        accounts.push(AccountMeta::new(*user_info_account, false));
        accounts.push(AccountMeta::new_readonly(*user_owner, true));
        accounts.push(AccountMeta::new(*user_lp_token_account, false));
        accounts.push(AccountMeta::new(*pool_lp_token_account, false));
        accounts.push(AccountMeta::new(*user_reward_token_account, false));
        accounts.push(AccountMeta::new(*pool_reward_token_account, false));
        accounts.push(AccountMeta::new_readonly(*clock_account, false));
        accounts.push(AccountMeta::new_readonly(*spl_token_program, false));

        Ok(Instruction {
            program_id: *stake_program_id,
            accounts,
            data,
        })
    }
    pub fn deposit_v4(
        stake_program_id: &Pubkey,
        pool_id: &Pubkey,
        pool_authority: &Pubkey,
        user_info_account: &Pubkey,
        user_owner: &Pubkey,
        user_lp_token_account: &Pubkey,
        pool_lp_token_account: &Pubkey,
        user_reward_token_account: &Pubkey,
        pool_reward_token_account: &Pubkey,
        clock_account: &Pubkey,
        spl_token_program: &Pubkey,
        user_reward_token_account_b: &Pubkey,
        pool_reward_token_account_b: &Pubkey,
        amount: u64,
    ) -> Result<Instruction, ProgramError> {
        let data = RaydiumInstruction::DepositV4 { amount }.pack();
        let mut accounts = Vec::with_capacity(12);
        accounts.push(AccountMeta::new(*pool_id, false));
        accounts.push(AccountMeta::new_readonly(*pool_authority, false));
        accounts.push(AccountMeta::new(*user_info_account, false));
        accounts.push(AccountMeta::new_readonly(*user_owner, true));
        accounts.push(AccountMeta::new(*user_lp_token_account, false));
        accounts.push(AccountMeta::new(*pool_lp_token_account, false));
        accounts.push(AccountMeta::new(*user_reward_token_account, false));
        accounts.push(AccountMeta::new(*pool_reward_token_account, false));
        accounts.push(AccountMeta::new_readonly(*clock_account, false));
        accounts.push(AccountMeta::new_readonly(*spl_token_program, false));
        accounts.push(AccountMeta::new(*user_reward_token_account_b, false));
        accounts.push(AccountMeta::new(*pool_reward_token_account_b, false));

        Ok(Instruction {
            program_id: *stake_program_id,
            accounts,
            data,
        })
    }
    pub fn harvest(
        stake_program_id: &Pubkey,
        pool_id: &Pubkey,
        pool_authority: &Pubkey,
        user_info_account: &Pubkey,
        user_owner: &Pubkey,
        user_lp_token_account: &Pubkey,
        pool_lp_token_account: &Pubkey,
        user_reward_token_account: &Pubkey,
        pool_reward_token_account: &Pubkey,
        clock_account: &Pubkey,
        spl_token_program: &Pubkey,
    ) -> Result<Instruction, ProgramError> {
        let data = RaydiumInstruction::Harvest {}.pack();
        let mut accounts = Vec::with_capacity(10);
        accounts.push(AccountMeta::new(*pool_id, false));
        accounts.push(AccountMeta::new_readonly(*pool_authority, false));
        accounts.push(AccountMeta::new(*user_info_account, false));
        accounts.push(AccountMeta::new_readonly(*user_owner, true));
        accounts.push(AccountMeta::new(*user_lp_token_account, false));
        accounts.push(AccountMeta::new(*pool_lp_token_account, false));
        accounts.push(AccountMeta::new(*user_reward_token_account, false));
        accounts.push(AccountMeta::new(*pool_reward_token_account, false));
        accounts.push(AccountMeta::new_readonly(*clock_account, false));
        accounts.push(AccountMeta::new_readonly(*spl_token_program, false));

        Ok(Instruction {
            program_id: *stake_program_id,
            accounts,
            data,
        })
    }
    pub fn harvest_v4(
        stake_program_id: &Pubkey,
        pool_id: &Pubkey,
        pool_authority: &Pubkey,
        user_info_account: &Pubkey,
        user_owner: &Pubkey,
        user_lp_token_account: &Pubkey,
        pool_lp_token_account: &Pubkey,
        user_reward_token_account: &Pubkey,
        pool_reward_token_account: &Pubkey,
        clock_account: &Pubkey,
        spl_token_program: &Pubkey,
        user_reward_token_account_b: &Pubkey,
        pool_reward_token_account_b: &Pubkey,
    ) -> Result<Instruction, ProgramError> {
        let data = RaydiumInstruction::HarvestV4 {}.pack();
        let mut accounts = Vec::with_capacity(12);
        accounts.push(AccountMeta::new(*pool_id, false));
        accounts.push(AccountMeta::new_readonly(*pool_authority, false));
        accounts.push(AccountMeta::new(*user_info_account, false));
        accounts.push(AccountMeta::new_readonly(*user_owner, true));
        accounts.push(AccountMeta::new(*user_lp_token_account, false));
        accounts.push(AccountMeta::new(*pool_lp_token_account, false));
        accounts.push(AccountMeta::new(*user_reward_token_account, false));
        accounts.push(AccountMeta::new(*pool_reward_token_account, false));
        accounts.push(AccountMeta::new_readonly(*clock_account, false));
        accounts.push(AccountMeta::new_readonly(*spl_token_program, false));
        accounts.push(AccountMeta::new(*user_reward_token_account_b, false));
        accounts.push(AccountMeta::new(*pool_reward_token_account_b, false));

        Ok(Instruction {
            program_id: *stake_program_id,
            accounts,
            data,
        })
    }
    pub fn withdraw(
        stake_program_id: &Pubkey,
        pool_id: &Pubkey,
        pool_authority: &Pubkey,
        user_info_account: &Pubkey,
        user_owner: &Pubkey,
        user_lp_token_account: &Pubkey,
        pool_lp_token_account: &Pubkey,
        user_reward_token_account: &Pubkey,
        pool_reward_token_account: &Pubkey,
        clock_account: &Pubkey,
        spl_token_program: &Pubkey,
        amount: u64,
    ) -> Result<Instruction, ProgramError> {
        let data = RaydiumInstruction::Withdraw { amount }.pack();
        let mut accounts = Vec::with_capacity(10);
        accounts.push(AccountMeta::new(*pool_id, false));
        accounts.push(AccountMeta::new_readonly(*pool_authority, false));
        accounts.push(AccountMeta::new(*user_info_account, false));
        accounts.push(AccountMeta::new_readonly(*user_owner, true));
        accounts.push(AccountMeta::new(*user_lp_token_account, false));
        accounts.push(AccountMeta::new(*pool_lp_token_account, false));
        accounts.push(AccountMeta::new(*user_reward_token_account, false));
        accounts.push(AccountMeta::new(*pool_reward_token_account, false));
        accounts.push(AccountMeta::new_readonly(*clock_account, false));
        accounts.push(AccountMeta::new_readonly(*spl_token_program, false));

        Ok(Instruction {
            program_id: *stake_program_id,
            accounts,
            data,
        })
    }
    pub fn withdraw_v4(
        stake_program_id: &Pubkey,
        pool_id: &Pubkey,
        pool_authority: &Pubkey,
        user_info_account: &Pubkey,
        user_owner: &Pubkey,
        user_lp_token_account: &Pubkey,
        pool_lp_token_account: &Pubkey,
        user_reward_token_account: &Pubkey,
        pool_reward_token_account: &Pubkey,
        clock_account: &Pubkey,
        spl_token_program: &Pubkey,
        user_reward_token_account_b: &Pubkey,
        pool_reward_token_account_b: &Pubkey,
        amount: u64,
    ) -> Result<Instruction, ProgramError> {
        let data = RaydiumInstruction::WithdrawV4 { amount }.pack();
        let mut accounts = Vec::with_capacity(12);
        accounts.push(AccountMeta::new(*pool_id, false));
        accounts.push(AccountMeta::new_readonly(*pool_authority, false));
        accounts.push(AccountMeta::new(*user_info_account, false));
        accounts.push(AccountMeta::new_readonly(*user_owner, true));
        accounts.push(AccountMeta::new(*user_lp_token_account, false));
        accounts.push(AccountMeta::new(*pool_lp_token_account, false));
        accounts.push(AccountMeta::new(*user_reward_token_account, false));
        accounts.push(AccountMeta::new(*pool_reward_token_account, false));
        accounts.push(AccountMeta::new_readonly(*clock_account, false));
        accounts.push(AccountMeta::new_readonly(*spl_token_program, false));
        accounts.push(AccountMeta::new(*user_reward_token_account_b, false));
        accounts.push(AccountMeta::new(*pool_reward_token_account_b, false));

        Ok(Instruction {
            program_id: *stake_program_id,
            accounts,
            data,
        })
    }
    pub fn swap(
        amm_program_id: &Pubkey,
        token_program_id: &Pubkey,
        amm_id: &Pubkey,
        amm_authority: &Pubkey,
        amm_open_orders: &Pubkey,
        amm_target_orders: &Pubkey,
        pool_coin_token_account: &Pubkey,
        pool_pc_token_account: &Pubkey,
        serum_program_id: &Pubkey,
        serum_market: &Pubkey,
        serum_bids: &Pubkey,
        serum_asks: &Pubkey,
        serum_event_queue: &Pubkey,
        serum_coin_vault_account: &Pubkey,
        serum_pc_vault_account: &Pubkey,
        serum_vault_signer: &Pubkey,
        user_source_token_account: &Pubkey,
        user_dest_token_account: &Pubkey,
        user_owner: &Pubkey,
        amount_in: u64,
        amount_out: u64,
    ) -> Result<Instruction, ProgramError> {
        let data = RaydiumInstruction::Swap {
            amount_in,
            amount_out,
        }
        .pack();
        let mut accounts = Vec::with_capacity(18);
        accounts.push(AccountMeta::new_readonly(*token_program_id, false));
        accounts.push(AccountMeta::new(*amm_id, false));
        accounts.push(AccountMeta::new_readonly(*amm_authority, false));
        accounts.push(AccountMeta::new(*amm_open_orders, false));
        accounts.push(AccountMeta::new(*amm_target_orders, false));
        accounts.push(AccountMeta::new(*pool_coin_token_account, false));
        accounts.push(AccountMeta::new(*pool_pc_token_account, false));
        accounts.push(AccountMeta::new_readonly(*serum_program_id, false));
        accounts.push(AccountMeta::new(*serum_market, false));
        accounts.push(AccountMeta::new(*serum_bids, false));
        accounts.push(AccountMeta::new(*serum_asks, false));
        accounts.push(AccountMeta::new(*serum_event_queue, false));
        accounts.push(AccountMeta::new(*serum_coin_vault_account, false));
        accounts.push(AccountMeta::new(*serum_pc_vault_account, false));
        accounts.push(AccountMeta::new_readonly(*serum_vault_signer, false));
        accounts.push(AccountMeta::new(*user_source_token_account, false));
        accounts.push(AccountMeta::new(*user_dest_token_account, false));
        accounts.push(AccountMeta::new_readonly(*user_owner, true));

        Ok(Instruction {
            program_id: *amm_program_id,
            accounts,
            data,
        })
    }
}
