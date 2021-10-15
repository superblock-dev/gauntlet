use crate::raydium::instruction::RaydiumInstruction;
use crate::utils::{check_pool_program_id, check_staking_program_id};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
};
pub struct Raydium;
impl Raydium {
    pub fn raydium_deposit(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let program_id = next_account_info(account_info_iter)?;
        let pool_id = next_account_info(account_info_iter)?;
        let pool_authority = next_account_info(account_info_iter)?;
        let user_info_account = next_account_info(account_info_iter)?;
        let user_owner = next_account_info(account_info_iter)?;
        let user_lp_token_account = next_account_info(account_info_iter)?;
        let pool_lp_token_account = next_account_info(account_info_iter)?;
        let user_reward_token_account = next_account_info(account_info_iter)?;
        let pool_reward_token_account = next_account_info(account_info_iter)?;
        let clock_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        check_staking_program_id(program_id).unwrap();
        let deposit_ix = RaydiumInstruction::deposit(
            program_id.key,
            pool_id.key,
            pool_authority.key,
            user_info_account.key,
            user_owner.key,
            user_lp_token_account.key,
            pool_lp_token_account.key,
            user_reward_token_account.key,
            pool_reward_token_account.key,
            clock_account.key,
            token_program.key,
            amount,
        )?;
        invoke_signed(&deposit_ix, accounts, &[&[&b"glt"[..], &[255]]])?;
        Ok(())
    }
    pub fn raydium_deposit_v4(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let program_id = next_account_info(account_info_iter)?;
        let pool_id = next_account_info(account_info_iter)?;
        let pool_authority = next_account_info(account_info_iter)?;
        let user_info_account = next_account_info(account_info_iter)?;
        let user_owner = next_account_info(account_info_iter)?;
        let user_lp_token_account = next_account_info(account_info_iter)?;
        let pool_lp_token_account = next_account_info(account_info_iter)?;
        let user_reward_token_account = next_account_info(account_info_iter)?;
        let pool_reward_token_account = next_account_info(account_info_iter)?;
        let clock_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let user_reward_token_account_b = next_account_info(account_info_iter)?;
        let pool_reward_token_account_b = next_account_info(account_info_iter)?;
        check_staking_program_id(program_id).unwrap();
        let deposit_v4_ix = RaydiumInstruction::deposit_v4(
            program_id.key,
            pool_id.key,
            pool_authority.key,
            user_info_account.key,
            user_owner.key,
            user_lp_token_account.key,
            pool_lp_token_account.key,
            user_reward_token_account.key,
            pool_reward_token_account.key,
            clock_account.key,
            token_program.key,
            user_reward_token_account_b.key,
            pool_reward_token_account_b.key,
            amount,
        )?;
        invoke_signed(&deposit_v4_ix, accounts, &[&[&b"glt"[..], &[255]]])?;
        Ok(())
    }
    pub fn raydium_harvest(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let program_id = next_account_info(account_info_iter)?;
        let pool_id = next_account_info(account_info_iter)?;
        let pool_authority = next_account_info(account_info_iter)?;
        let user_info_account = next_account_info(account_info_iter)?;
        let user_owner = next_account_info(account_info_iter)?;
        let user_lp_token_account = next_account_info(account_info_iter)?;
        let pool_lp_token_account = next_account_info(account_info_iter)?;
        let user_reward_token_account = next_account_info(account_info_iter)?;
        let pool_reward_token_account = next_account_info(account_info_iter)?;
        let clock_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        check_staking_program_id(program_id).unwrap();
        let harvest_ix = RaydiumInstruction::harvest(
            program_id.key,
            pool_id.key,
            pool_authority.key,
            user_info_account.key,
            user_owner.key,
            user_lp_token_account.key,
            pool_lp_token_account.key,
            user_reward_token_account.key,
            pool_reward_token_account.key,
            clock_account.key,
            token_program.key,
        )?;
        invoke_signed(&harvest_ix, accounts, &[&[&b"glt"[..], &[255]]])?;
        Ok(())
    }
    pub fn raydium_harvest_v4(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let program_id = next_account_info(account_info_iter)?;
        let pool_id = next_account_info(account_info_iter)?;
        let pool_authority = next_account_info(account_info_iter)?;
        let user_info_account = next_account_info(account_info_iter)?;
        let user_owner = next_account_info(account_info_iter)?;
        let user_lp_token_account = next_account_info(account_info_iter)?;
        let pool_lp_token_account = next_account_info(account_info_iter)?;
        let user_reward_token_account = next_account_info(account_info_iter)?;
        let pool_reward_token_account = next_account_info(account_info_iter)?;
        let clock_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let user_reward_token_account_b = next_account_info(account_info_iter)?;
        let pool_reward_token_account_b = next_account_info(account_info_iter)?;
        check_staking_program_id(program_id).unwrap();
        let deposit_v4_ix = RaydiumInstruction::harvest_v4(
            program_id.key,
            pool_id.key,
            pool_authority.key,
            user_info_account.key,
            user_owner.key,
            user_lp_token_account.key,
            pool_lp_token_account.key,
            user_reward_token_account.key,
            pool_reward_token_account.key,
            clock_account.key,
            token_program.key,
            user_reward_token_account_b.key,
            pool_reward_token_account_b.key,
        )?;
        invoke_signed(&deposit_v4_ix, accounts, &[&[&b"glt"[..], &[255]]])?;
        Ok(())
    }
    pub fn raydium_withdraw(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let program_id = next_account_info(account_info_iter)?;
        let pool_id = next_account_info(account_info_iter)?;
        let pool_authority = next_account_info(account_info_iter)?;
        let user_info_account = next_account_info(account_info_iter)?;
        let user_owner = next_account_info(account_info_iter)?;
        let user_lp_token_account = next_account_info(account_info_iter)?;
        let pool_lp_token_account = next_account_info(account_info_iter)?;
        let user_reward_token_account = next_account_info(account_info_iter)?;
        let pool_reward_token_account = next_account_info(account_info_iter)?;
        let clock_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        check_staking_program_id(program_id).unwrap();
        let withdraw_ix = RaydiumInstruction::withdraw(
            program_id.key,
            pool_id.key,
            pool_authority.key,
            user_info_account.key,
            user_owner.key,
            user_lp_token_account.key,
            pool_lp_token_account.key,
            user_reward_token_account.key,
            pool_reward_token_account.key,
            clock_account.key,
            token_program.key,
            amount,
        )?;
        invoke_signed(&withdraw_ix, accounts, &[&[&b"glt"[..], &[255]]])?;
        Ok(())
    }
    pub fn raydium_withdraw_v4(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let program_id = next_account_info(account_info_iter)?;
        let pool_id = next_account_info(account_info_iter)?;
        let pool_authority = next_account_info(account_info_iter)?;
        let user_info_account = next_account_info(account_info_iter)?;
        let user_owner = next_account_info(account_info_iter)?;
        let user_lp_token_account = next_account_info(account_info_iter)?;
        let pool_lp_token_account = next_account_info(account_info_iter)?;
        let user_reward_token_account = next_account_info(account_info_iter)?;
        let pool_reward_token_account = next_account_info(account_info_iter)?;
        let clock_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let user_reward_token_account_b = next_account_info(account_info_iter)?;
        let pool_reward_token_account_b = next_account_info(account_info_iter)?;
        check_staking_program_id(program_id).unwrap();
        let withdraw_v4_ix = RaydiumInstruction::withdraw_v4(
            program_id.key,
            pool_id.key,
            pool_authority.key,
            user_info_account.key,
            user_owner.key,
            user_lp_token_account.key,
            pool_lp_token_account.key,
            user_reward_token_account.key,
            pool_reward_token_account.key,
            clock_account.key,
            token_program.key,
            user_reward_token_account_b.key,
            pool_reward_token_account_b.key,
            amount,
        )?;
        invoke_signed(&withdraw_v4_ix, accounts, &[&[&b"glt"[..], &[255]]])?;
        Ok(())
    }
    pub fn raydium_swap(
        accounts: &[AccountInfo],
        amount_in: u64,
        amount_out: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let amm_program_id = next_account_info(account_info_iter)?;
        let token_program_id = next_account_info(account_info_iter)?;
        let amm_id = next_account_info(account_info_iter)?;
        let amm_authority = next_account_info(account_info_iter)?;
        let amm_open_orders = next_account_info(account_info_iter)?;
        let amm_target_orders = next_account_info(account_info_iter)?;
        let pool_coin_token_account = next_account_info(account_info_iter)?;
        let pool_pc_token_account = next_account_info(account_info_iter)?;
        let serum_program_id = next_account_info(account_info_iter)?;
        let serum_market = next_account_info(account_info_iter)?;
        let serum_bids = next_account_info(account_info_iter)?;
        let serum_asks = next_account_info(account_info_iter)?;
        let serum_event_queue = next_account_info(account_info_iter)?;
        let serum_coin_vault_account = next_account_info(account_info_iter)?;
        let serum_pc_vault_account = next_account_info(account_info_iter)?;
        let serum_vault_signer = next_account_info(account_info_iter)?;
        let user_source_token_account = next_account_info(account_info_iter)?;
        let user_dest_token_account = next_account_info(account_info_iter)?;
        let user_owner = next_account_info(account_info_iter)?;
        check_pool_program_id(amm_program_id).unwrap();
        let swap_ix = RaydiumInstruction::swap(
            amm_program_id.key,
            token_program_id.key,
            amm_id.key,
            amm_authority.key,
            amm_open_orders.key,
            amm_target_orders.key,
            pool_coin_token_account.key,
            pool_pc_token_account.key,
            serum_program_id.key,
            serum_market.key,
            serum_bids.key,
            serum_asks.key,
            serum_event_queue.key,
            serum_coin_vault_account.key,
            serum_pc_vault_account.key,
            serum_vault_signer.key,
            user_source_token_account.key,
            user_dest_token_account.key,
            user_owner.key,
            amount_in,
            amount_out,
        )?;
        invoke_signed(&swap_ix, accounts, &[&[&b"glt"[..], &[255]]])?;
        Ok(())
    }
}
