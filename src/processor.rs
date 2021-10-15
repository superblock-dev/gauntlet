use std::time::Duration;

use solana_program::{
    account_info::{next_account_info, next_account_infos, AccountInfo},
    clock::{Clock, UnixTimestamp},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use spl_token::state::Account;

use crate::{
    error::GauntletError,
    instruction::{DepositType, GauntletInstruction, SwapType, WithdrawType},
    raydium::raydium::Raydium,
    state::{Fees, Gauntlet, Status, Strategy, User, Vault, VaultStrategy},
    utils::{
        change_token_account_owner, create_pda_account, transfer_token, transfer_token_signed,
    },
};

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instructions = GauntletInstruction::unpack(instruction_data)?;
        match instructions {
            GauntletInstruction::InitGauntlet {} => Self::init_gauntlet(accounts, program_id),
            GauntletInstruction::InitVault { fees } => Self::init_vault(accounts, fees, program_id),
            GauntletInstruction::InitStrategy {} => Self::init_strategy(accounts, program_id),
            GauntletInstruction::UpdateVaultStrategy {
                availability,
                needs_usdc_pool,
            } => Self::update_vault_strategy(accounts, availability, needs_usdc_pool),
            GauntletInstruction::Deposit {
                amount,
                deposit_type,
            } => Self::deposit(accounts, amount, deposit_type),
            GauntletInstruction::Harvest { deposit_type } => Self::harvest(accounts, deposit_type),
            GauntletInstruction::SwapFarmRewardToUsdc { swap_type } => {
                Self::swap_farm_reward_to_usdc(accounts, swap_type)
            }
            GauntletInstruction::SwapUsdcToStrategyToken { swap_type } => {
                Self::swap_usdc_to_strategy_token(accounts, swap_type)
            }
            GauntletInstruction::SwapFarmRewardToStrategyToken { swap_type } => {
                Self::swap_reward_to_strategy_token(accounts, swap_type)
            }
            GauntletInstruction::Withdraw {
                amount,
                reward_amount,
                withdraw_type,
            } => Self::withdraw(accounts, amount, reward_amount, withdraw_type),
            GauntletInstruction::CreateUserAccount {} => {
                Self::create_user_account(accounts, program_id)
            }
        }
    }
    fn init_gauntlet(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;
        let gauntlet_state_account = next_account_info(account_info_iter)?;
        let usdc_token_account = next_account_info(account_info_iter)?;
        let _token_program_account = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut gauntlet_info = Gauntlet::unpack_unchecked(&gauntlet_state_account.data.borrow())?;

        if gauntlet_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        gauntlet_info = Gauntlet::init(*initializer.key, *usdc_token_account.key);

        Gauntlet::pack(gauntlet_info, &mut gauntlet_state_account.data.borrow_mut())?;

        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"glt"], program_id); // TODO change
        change_token_account_owner(usdc_token_account, initializer, &pda)?;

        Ok(())
    }

    fn init_vault(accounts: &[AccountInfo], fees: Fees, program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;
        let gauntlet_state_account = next_account_info(account_info_iter)?;
        let vault_state_account = next_account_info(account_info_iter)?;
        let vault_strategy_account = next_account_info(account_info_iter)?;
        let deposit_token_account = next_account_info(account_info_iter)?;
        let withdraw_fee_token_account = next_account_info(account_info_iter)?;
        let vault_raydium_state_account = next_account_info(account_info_iter)?;
        let raydium_staking_program = next_account_info(account_info_iter)?;
        let _token_program_account = next_account_info(account_info_iter)?;
        let system_program_account = next_account_info(account_info_iter)?;
        let farm_reward_token_account = next_account_info(account_info_iter)?;
        let mut farm_second_reward_token_account: Option<&AccountInfo> = None;

        if accounts.len() > 11 {
            farm_second_reward_token_account = Some(next_account_info(account_info_iter)?);
        }

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut gauntlet_info = Gauntlet::unpack(&gauntlet_state_account.data.borrow())?;

        if gauntlet_info.admin != *initializer.key {
            return Err(GauntletError::NotAdmin.into());
        }

        let mut vault_info = Vault::unpack_unchecked(&vault_state_account.data.borrow())?;

        if vault_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        let farm_reward_token_account_info =
            Account::unpack(&farm_reward_token_account.data.borrow())?;
        vault_info.is_initialized = true;
        vault_info.index = gauntlet_info.vaults_len;
        vault_info.status = Status::default();
        vault_info.fees = fees;
        vault_info.gauntlet_state_account = *gauntlet_state_account.key;
        vault_info.deposit_token_account = *deposit_token_account.key;
        vault_info.reward_token_account = *farm_reward_token_account.key;
        gauntlet_info.vaults_len = gauntlet_info.vaults_len.checked_add(1).unwrap();

        if farm_second_reward_token_account.is_some() {
            let farm_second_reward_token_account_unwrapped =
                farm_second_reward_token_account.unwrap();
            let farm_second_reward_token_account_info =
                Account::unpack(&farm_second_reward_token_account_unwrapped.data.borrow())?;
            if farm_second_reward_token_account_info.mint == farm_reward_token_account_info.mint {
                // reward token 과 reward token b 가 같으면 에러
                return Err(GauntletError::DuplicateFarmRewardToken.into());
            } else {
                vault_info.reward_token_b_account = *farm_second_reward_token_account_unwrapped.key;
            }
        }
        vault_info.withdraw_fee_account = *withdraw_fee_token_account.key;
        vault_info.last_reward_update_time = 0;
        vault_info.total_deposit_amount = 0;
        let (_pda, _seed) = Pubkey::find_program_address(
            &[
                &gauntlet_state_account.key.to_bytes(),
                &vault_state_account.key.to_bytes(),
                &vault_strategy_account.key.to_bytes(),
            ],
            program_id,
        );
        if *vault_raydium_state_account.key != _pda {
            return Err(ProgramError::InvalidSeeds);
        }
        // create raydium state account
        let data_size = match farm_second_reward_token_account.is_some() {
            true => 96,
            false => 88,
        };
        create_pda_account(
            initializer,
            data_size,
            raydium_staking_program.key,
            system_program_account,
            vault_raydium_state_account,
            &[
                &gauntlet_state_account.key.to_bytes(),
                &vault_state_account.key.to_bytes(),
                &vault_strategy_account.key.to_bytes(),
                &[_seed],
            ],
        )?;

        vault_info.raydium_state_account = *vault_raydium_state_account.key;
        Vault::pack(vault_info, &mut vault_state_account.data.borrow_mut())?;
        Gauntlet::pack(gauntlet_info, &mut gauntlet_state_account.data.borrow_mut())?;

        let mut vault_strategy_info =
            VaultStrategy::unpack_unchecked(&vault_strategy_account.data.borrow())?;

        if vault_strategy_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        vault_strategy_info = VaultStrategy::init(*vault_state_account.key);

        VaultStrategy::pack(
            vault_strategy_info,
            &mut vault_strategy_account.data.borrow_mut(),
        )?;

        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"glt"], program_id);

        change_token_account_owner(deposit_token_account, initializer, &pda)?;

        change_token_account_owner(farm_reward_token_account, initializer, &pda)?;

        if farm_second_reward_token_account.is_some() {
            change_token_account_owner(
                farm_second_reward_token_account.unwrap(),
                initializer,
                &pda,
            )?;
        }

        Ok(())
    }

    fn init_strategy(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin = next_account_info(account_info_iter)?;
        let gauntlet_state_account = next_account_info(account_info_iter)?;
        let strategy_state_account = next_account_info(account_info_iter)?;
        let strategy_token_account = next_account_info(account_info_iter)?;
        let performance_fee_token_account = next_account_info(account_info_iter)?;
        let _token_program_account = next_account_info(account_info_iter)?;

        if !admin.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut gauntlet_info = Gauntlet::unpack(&gauntlet_state_account.data.borrow())?;

        if gauntlet_info.admin != *admin.key {
            return Err(GauntletError::NotAdmin.into());
        }

        let mut strategy_info = Strategy::unpack_unchecked(&strategy_state_account.data.borrow())?;

        if strategy_info.is_initialized {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        strategy_info = Strategy::init(
            gauntlet_info.strategies_len,
            *gauntlet_state_account.key,
            *admin.key,
            *performance_fee_token_account.key,
            *strategy_token_account.key,
        );
        gauntlet_info.strategies_len = gauntlet_info.strategies_len.checked_add(1).unwrap();

        Strategy::pack(strategy_info, &mut strategy_state_account.data.borrow_mut())?;
        Gauntlet::pack(gauntlet_info, &mut gauntlet_state_account.data.borrow_mut())?;

        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"glt"], program_id); // TODO CHANGE

        change_token_account_owner(strategy_token_account, admin, &pda)?;

        Ok(())
    }

    fn update_vault_strategy(
        accounts: &[AccountInfo],
        availability: bool,
        needs_usdc_pool: bool,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin = next_account_info(account_info_iter)?;
        let gauntlet_state_account = next_account_info(account_info_iter)?;
        let vault_strategy_state_account = next_account_info(account_info_iter)?;
        let vault_state_account = next_account_info(account_info_iter)?;
        let strategy_state_account = next_account_info(account_info_iter)?;

        if !admin.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let gauntlet_info = Gauntlet::unpack(&gauntlet_state_account.data.borrow())?;

        if gauntlet_info.admin != *admin.key {
            return Err(GauntletError::NotAdmin.into());
        }
        let mut vault_strategy_info =
            VaultStrategy::unpack(&vault_strategy_state_account.data.borrow())?;
        let strategy_info = Strategy::unpack(&strategy_state_account.data.borrow())?;
        let mut vault_info = Vault::unpack(&vault_state_account.data.borrow())?;

        vault_strategy_info.needs_usdc_pools[strategy_info.index as usize] = needs_usdc_pool;
        vault_strategy_info.availabilities[strategy_info.index as usize] = availability;
        if vault_info.deposit_amounts[strategy_info.index as usize] > 0 {
            // flag 에 따라서 valid 한 total deposit amount를 설정해줌
            if availability {
                vault_info.total_deposit_amount = vault_info
                    .total_deposit_amount
                    .checked_add(vault_info.deposit_amounts[strategy_info.index as usize])
                    .unwrap();
            } else {
                vault_info.total_deposit_amount = vault_info
                    .total_deposit_amount
                    .checked_sub(vault_info.deposit_amounts[strategy_info.index as usize])
                    .unwrap();
            }
        }

        Vault::pack(vault_info, &mut vault_state_account.data.borrow_mut())?;

        VaultStrategy::pack(
            vault_strategy_info,
            &mut vault_strategy_state_account.data.borrow_mut(),
        )?;
        Ok(())
    }
    fn harvest(accounts: &[AccountInfo], deposit_type: DepositType) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let harvestor = next_account_info(account_info_iter)?; // signer
        let gauntlet_state_account = next_account_info(account_info_iter)?;
        let harvestor_user_state_account = next_account_info(account_info_iter)?;
        let vault_state_account = next_account_info(account_info_iter)?;
        let vault_strategy_state_account = next_account_info(account_info_iter)?;
        let harvest_accounts = match deposit_type {
            DepositType::RAYDIUM => next_account_infos(account_info_iter, 11).unwrap(),
            DepositType::RAYDIUM_V4 => next_account_infos(account_info_iter, 13).unwrap(),
        };
        let vault_deposit_token_account = &harvest_accounts[5];
        let vault_reward_token_account = &harvest_accounts[7];
        let vault_reward_b_token_account = match deposit_type {
            DepositType::RAYDIUM => None,
            DepositType::RAYDIUM_V4 => Some(&harvest_accounts[11]),
        };
        let gauntlet_info = Gauntlet::unpack(&gauntlet_state_account.data.borrow())?;
        let mut vault_info = Vault::unpack(&vault_state_account.data.borrow())?;
        let vault_strategy_info =
            VaultStrategy::unpack(&vault_strategy_state_account.data.borrow())?;
        let mut harvestor_user_info =
            User::unpack_unchecked(&harvestor_user_state_account.data.borrow())?;
        let clock = &Clock::get()?;
        if !harvestor.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if vault_info.gauntlet_state_account != *gauntlet_state_account.key {
            return Err(GauntletError::WrongVaultStateAccount.into());
        }

        if vault_strategy_info.vault_account != *vault_state_account.key {
            return Err(GauntletError::WrongVaultStrategyStateAccount.into());
        }

        if vault_info.deposit_token_account != *vault_deposit_token_account.key {
            return Err(GauntletError::WrongTokenAccount.into());
        }

        if vault_info.reward_token_account != *vault_reward_token_account.key {
            return Err(GauntletError::WrongTokenAccount.into());
        }

        if vault_reward_b_token_account.is_some() {
            if vault_info.reward_token_b_account != *vault_reward_b_token_account.unwrap().key {
                return Err(GauntletError::WrongTokenAccount.into());
            }
        }

        if vault_info.total_deposit_amount > 0 {
            Self::_harvest(
                &gauntlet_info,
                &mut vault_info,
                &vault_strategy_info,
                harvest_accounts,
                &vault_reward_token_account,
                &vault_reward_b_token_account,
                &deposit_type,
            )
            .unwrap();
        }

        harvestor_user_info.user_status = 1;
        harvestor_user_info.deadline = clock
            .unix_timestamp
            .checked_add(Duration::from_secs(30).as_secs() as UnixTimestamp)
            .unwrap();

        User::pack(
            harvestor_user_info,
            &mut harvestor_user_state_account.data.borrow_mut(),
        )?;
        Vault::pack(vault_info, &mut vault_state_account.data.borrow_mut())?;

        Ok(())
    }

    fn swap_farm_reward_to_usdc(accounts: &[AccountInfo], swap_type: SwapType) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let swaper = next_account_info(account_info_iter)?; // signer
        let gauntlet_state_account = next_account_info(account_info_iter)?;
        let swaper_user_state_account = next_account_info(account_info_iter)?;
        let vault_state_account = next_account_info(account_info_iter)?;
        let vault_strategy_state_account = next_account_info(account_info_iter)?;
        let strategy_state_account = next_account_info(account_info_iter)?;
        let swap_reward_to_usdc_accounts = match swap_type {
            SwapType::RAYDIUM => next_account_infos(account_info_iter, 19).unwrap(),
        };
        let vault_reward_token_account = &swap_reward_to_usdc_accounts[16];
        let gauntlet_usdc_token_account = &swap_reward_to_usdc_accounts[17];
        let mut swaper_user_info =
            User::unpack_unchecked(&swaper_user_state_account.data.borrow())?;
        let gauntlet_info = Gauntlet::unpack(&gauntlet_state_account.data.borrow())?;
        let mut vault_info = Vault::unpack(&vault_state_account.data.borrow())?;
        let vault_strategy_info =
            VaultStrategy::unpack(&vault_strategy_state_account.data.borrow())?;
        let strategy_info = Strategy::unpack(&strategy_state_account.data.borrow())?;
        let strategy_index = strategy_info.index as usize;
        let mut second_reward_token = false;
        let clock = &Clock::get()?;

        if !swaper.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if *gauntlet_state_account.key != vault_info.gauntlet_state_account {
            return Err(GauntletError::WrongVaultStateAccount.into());
        }

        if *vault_state_account.key != vault_strategy_info.vault_account {
            return Err(GauntletError::WrongVaultStrategyStateAccount.into());
        }

        if vault_strategy_info.needs_usdc_pools[strategy_index] == false {
            return Err(GauntletError::WrongVaultStrategyStateAccount.into());
        }

        if *gauntlet_state_account.key != strategy_info.gauntlet_state_account {
            return Err(GauntletError::WrongStrategyStateAccount.into());
        }

        if gauntlet_info.usdc_token_account != *gauntlet_usdc_token_account.key {
            return Err(GauntletError::WrongTokenAccount.into());
        }

        if *vault_reward_token_account.key == vault_info.reward_token_b_account {
            second_reward_token = true;
        } else if *vault_reward_token_account.key != vault_info.reward_token_account {
            return Err(GauntletError::RewardTokenAccountError.into());
        }

        if !second_reward_token && swaper_user_info.user_status != 1 {
            return Err(GauntletError::UserStatusError.into());
        }
        if second_reward_token && swaper_user_info.user_status != 2 {
            return Err(GauntletError::UserStatusError.into());
        }

        if clock.unix_timestamp > swaper_user_info.deadline {
            return Err(GauntletError::TimeoutError.into());
        }

        if vault_strategy_info.vault_account != *vault_state_account.key {
            return Err(GauntletError::WrongVaultStateAccount.into());
        }

        if vault_strategy_info.availabilities[strategy_index] {
            // 해당 vault와 strategy가 available할때만 swap, available하지않으면 harvest만 하고 swap은 하지않음
            Self::_swap_farm_token_to_usdc(
                &mut vault_info,
                strategy_index,
                gauntlet_usdc_token_account,
                swap_reward_to_usdc_accounts,
                &swap_type,
                second_reward_token,
            )
            .unwrap();
        }
        if vault_info.reward_token_b_account == Pubkey::default() {
            swaper_user_info.user_status += 2;
        } else {
            swaper_user_info.user_status += 1;
        }
        swaper_user_info.deadline = clock
            .unix_timestamp
            .checked_add(Duration::from_secs(30).as_secs() as UnixTimestamp)
            .unwrap();
        User::pack(
            swaper_user_info,
            &mut swaper_user_state_account.data.borrow_mut(),
        )?;
        Vault::pack(vault_info, &mut vault_state_account.data.borrow_mut())?;

        Ok(())
    }

    fn swap_usdc_to_strategy_token(accounts: &[AccountInfo], swap_type: SwapType) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let swaper = next_account_info(account_info_iter)?; // signer
        let gauntlet_state_account = next_account_info(account_info_iter)?;
        let swaper_user_state_account = next_account_info(account_info_iter)?;
        let vault_state_account = next_account_info(account_info_iter)?;
        let vault_strategy_state_account = next_account_info(account_info_iter)?;
        let strategy_state_account = next_account_info(account_info_iter)?;
        let swap_usdc_to_strategy_accounts = match swap_type {
            SwapType::RAYDIUM => next_account_infos(account_info_iter, 19).unwrap(),
        };
        let gauntlet_usdc_token_account = &swap_usdc_to_strategy_accounts[16];
        let strategy_token_account = &swap_usdc_to_strategy_accounts[17];
        let gauntlet_info = Gauntlet::unpack(&gauntlet_state_account.data.borrow())?;
        let mut swaper_user_info =
            User::unpack_unchecked(&swaper_user_state_account.data.borrow())?;
        let mut vault_info = Vault::unpack(&vault_state_account.data.borrow())?;
        let mut vault_strategy_info =
            VaultStrategy::unpack(&vault_strategy_state_account.data.borrow())?;
        let mut strategy_info = Strategy::unpack(&strategy_state_account.data.borrow())?;
        let strategy_index = strategy_info.index as usize;
        let clock = &Clock::get()?;

        if !swaper.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if *gauntlet_state_account.key != vault_info.gauntlet_state_account {
            return Err(GauntletError::WrongVaultStateAccount.into());
        }

        if *vault_state_account.key != vault_strategy_info.vault_account {
            return Err(GauntletError::WrongVaultStrategyStateAccount.into());
        }
        if vault_strategy_info.needs_usdc_pools[strategy_index] == false {
            return Err(GauntletError::WrongVaultStrategyStateAccount.into());
        }

        if *gauntlet_state_account.key != strategy_info.gauntlet_state_account {
            return Err(GauntletError::WrongStrategyStateAccount.into());
        }

        if gauntlet_info.usdc_token_account != *gauntlet_usdc_token_account.key {
            return Err(GauntletError::WrongTokenAccount.into());
        }

        if strategy_info.strategy_token_account != *strategy_token_account.key {
            return Err(GauntletError::WrongTokenAccount.into());
        }

        if vault_strategy_info.vault_account != *vault_state_account.key {
            return Err(GauntletError::WrongVaultStateAccount.into());
        }

        if swaper_user_info.user_status != 3 {
            return Err(GauntletError::UserStatusError.into());
        }

        if clock.unix_timestamp > swaper_user_info.deadline {
            return Err(GauntletError::TimeoutError.into());
        }
        if vault_strategy_info.availabilities[strategy_index]
            && vault_info.deposit_amounts[strategy_index] != 0
        {
            Self::_swap_usdc_to_strategy_token(
                &mut vault_info,
                &mut vault_strategy_info,
                &mut strategy_info,
                strategy_token_account,
                gauntlet_usdc_token_account,
                swap_usdc_to_strategy_accounts,
                &swap_type,
            )
            .unwrap();
        }
        swaper_user_info.user_status += 1;
        swaper_user_info.deadline = clock
            .unix_timestamp
            .checked_add(Duration::from_secs(30).as_secs() as UnixTimestamp)
            .unwrap();
        User::pack(
            swaper_user_info,
            &mut swaper_user_state_account.data.borrow_mut(),
        )?;
        Vault::pack(vault_info, &mut vault_state_account.data.borrow_mut())?;
        VaultStrategy::pack(
            vault_strategy_info,
            &mut vault_strategy_state_account.data.borrow_mut(),
        )?;
        Strategy::pack(strategy_info, &mut strategy_state_account.data.borrow_mut())?;

        Ok(())
    }

    fn swap_reward_to_strategy_token(
        accounts: &[AccountInfo],
        swap_type: SwapType,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let swaper = next_account_info(account_info_iter)?; // signer
        let gauntlet_state_account = next_account_info(account_info_iter)?;
        let swaper_user_state_account = next_account_info(account_info_iter)?;
        let vault_state_account = next_account_info(account_info_iter)?;
        let vault_strategy_state_account = next_account_info(account_info_iter)?;
        let strategy_state_account = next_account_info(account_info_iter)?;
        let swap_reward_to_strategy_accounts = match swap_type {
            SwapType::RAYDIUM => next_account_infos(account_info_iter, 19).unwrap(),
        };
        let vault_reward_token_account = &swap_reward_to_strategy_accounts[16];
        let strategy_token_account = &swap_reward_to_strategy_accounts[17];
        let mut swaper_user_info =
            User::unpack_unchecked(&swaper_user_state_account.data.borrow())?;
        let mut vault_info = Vault::unpack(&vault_state_account.data.borrow())?;
        let mut vault_strategy_info =
            VaultStrategy::unpack(&vault_strategy_state_account.data.borrow())?;
        let mut strategy_info = Strategy::unpack(&strategy_state_account.data.borrow())?;
        let strategy_index = strategy_info.index as usize;
        let mut second_reward_token = false;
        let clock = &Clock::get()?;

        if !swaper.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if *gauntlet_state_account.key != vault_info.gauntlet_state_account {
            return Err(GauntletError::WrongVaultStateAccount.into());
        }

        if *vault_state_account.key != vault_strategy_info.vault_account {
            return Err(GauntletError::WrongVaultStrategyStateAccount.into());
        }
        if vault_strategy_info.needs_usdc_pools[strategy_index] == true {
            return Err(GauntletError::WrongVaultStrategyStateAccount.into());
        }

        if *gauntlet_state_account.key != strategy_info.gauntlet_state_account {
            return Err(GauntletError::WrongStrategyStateAccount.into());
        }

        if *vault_reward_token_account.key == vault_info.reward_token_b_account {
            second_reward_token = true;
        } else if *vault_reward_token_account.key != vault_info.reward_token_account {
            return Err(GauntletError::RewardTokenAccountError.into());
        }
        if strategy_info.strategy_token_account != *strategy_token_account.key {
            return Err(GauntletError::WrongTokenAccount.into());
        }

        if vault_strategy_info.vault_account != *vault_state_account.key {
            return Err(GauntletError::WrongVaultStateAccount.into());
        }

        if !second_reward_token && swaper_user_info.user_status != 1 {
            return Err(GauntletError::UserStatusError.into());
        }
        if second_reward_token && swaper_user_info.user_status != 2 {
            return Err(GauntletError::UserStatusError.into());
        }
        if clock.unix_timestamp > swaper_user_info.deadline {
            return Err(GauntletError::TimeoutError.into());
        }
        if vault_strategy_info.availabilities[strategy_index]
            && vault_info.deposit_amounts[strategy_index] != 0
        {
            Self::_swap_reward_to_strategy_token(
                &mut vault_info,
                &mut vault_strategy_info,
                &mut strategy_info,
                strategy_token_account,
                swap_reward_to_strategy_accounts,
                &swap_type,
                second_reward_token,
            )
            .unwrap();
        }
        if vault_info.reward_token_b_account == Pubkey::default() {
            swaper_user_info.user_status = 4;
        } else if vault_info.reward_token_b_account == *vault_reward_token_account.key {
            swaper_user_info.user_status = 4;
        } else {
            swaper_user_info.user_status += 1;
        }
        swaper_user_info.deadline = clock
            .unix_timestamp
            .checked_add(Duration::from_secs(30).as_secs() as UnixTimestamp)
            .unwrap();
        User::pack(
            swaper_user_info,
            &mut swaper_user_state_account.data.borrow_mut(),
        )?;
        Vault::pack(vault_info, &mut vault_state_account.data.borrow_mut())?;
        VaultStrategy::pack(
            vault_strategy_info,
            &mut vault_strategy_state_account.data.borrow_mut(),
        )?;
        Strategy::pack(strategy_info, &mut strategy_state_account.data.borrow_mut())?;

        Ok(())
    }

    fn raydium_swap(accounts: &[AccountInfo], amount_in: u64, amount_out: u64) -> ProgramResult {
        // let pda = *accounts[18].key;
        // let pda_address = Pubkey::from_str("KP2AwjL3wwpZcy37wiiDVS4qaVhYP4tU2xTunvWp2ut").unwrap();
        // assert_eq!(pda, pda_address);
        // let token_a_info = Account::unpack(&accounts[16].data.borrow())?;
        // let token_b_info = Account::unpack(&accounts[17].data.borrow())?;
        // assert_eq!(token_a_info.owner, pda_address);
        // assert_eq!(token_b_info.owner, pda_address);
        let pool_coin_token_account_info = Account::unpack(&accounts[6].data.borrow())?;
        let pool_pc_token_account_info = Account::unpack(&accounts[7].data.borrow())?;
        let source_token_account_info = Account::unpack(&accounts[16].data.borrow())?;
        let dest_token_amount;
        if pool_coin_token_account_info.mint == source_token_account_info.mint {
            dest_token_amount = (pool_pc_token_account_info.amount as u128)
                .checked_mul(source_token_account_info.amount as u128)
                .unwrap()
                .checked_div(pool_coin_token_account_info.amount as u128)
                .unwrap() as u64;
        } else {
            dest_token_amount = (pool_coin_token_account_info.amount as u128)
                .checked_mul(source_token_account_info.amount as u128)
                .unwrap()
                .checked_div(pool_pc_token_account_info.amount as u128)
                .unwrap() as u64;
        }
        if dest_token_amount >= 20 {
            Raydium::raydium_swap(accounts, amount_in, amount_out).unwrap();
        }
        Ok(())
    }

    fn deposit(accounts: &[AccountInfo], amount: u64, deposit_type: DepositType) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let depositor = next_account_info(account_info_iter)?;
        let depositor_user_state_account = next_account_info(account_info_iter)?;
        let depositor_deposit_token_account = next_account_info(account_info_iter)?;
        let gauntlet_state_account = next_account_info(account_info_iter)?;
        let vault_state_account = next_account_info(account_info_iter)?;
        let vault_strategy_state_account = next_account_info(account_info_iter)?;
        let strategy_account = next_account_info(account_info_iter)?;
        let deposit_accounts = match deposit_type {
            DepositType::RAYDIUM => next_account_infos(account_info_iter, 11).unwrap(),
            DepositType::RAYDIUM_V4 => next_account_infos(account_info_iter, 13).unwrap(),
        };
        let vault_deposit_token_account = &deposit_accounts[5];
        let vault_reward_token_account = &deposit_accounts[7];
        let vault_reward_b_token_account = match deposit_type {
            DepositType::RAYDIUM => None,
            DepositType::RAYDIUM_V4 => Some(&deposit_accounts[11]),
        };

        let mut depositor_user_info =
            User::unpack_unchecked(&depositor_user_state_account.data.borrow())?;
        let depositor_token_account_info =
            Account::unpack(&depositor_deposit_token_account.data.borrow())?;
        let mut vault_info = Vault::unpack(&vault_state_account.data.borrow())?;
        let vault_deposit_token_account_info =
            Account::unpack(&vault_deposit_token_account.data.borrow())?;
        let vault_strategy_info =
            VaultStrategy::unpack(&vault_strategy_state_account.data.borrow())?;
        let strategy_info = Strategy::unpack(&strategy_account.data.borrow())?;
        let strategy_index = strategy_info.index as usize;

        if !depositor_user_info.is_initialized {
            depositor_user_info.is_initialized = true;
            depositor_user_info.user = *depositor.key;
            depositor_user_info.vault_account = *vault_state_account.key;
            depositor_user_info.strategy_account = *strategy_account.key;
            depositor_user_info.amount = 0;
        }

        if !depositor.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if *depositor.key != depositor_user_info.user {
            return Err(GauntletError::WrongUserAccount.into());
        }

        if *vault_state_account.key != depositor_user_info.vault_account {
            return Err(GauntletError::WrongVaultStateAccount.into());
        }

        if *strategy_account.key != depositor_user_info.strategy_account {
            return Err(GauntletError::WrongUserAccount.into());
        }

        if depositor_token_account_info.mint != vault_deposit_token_account_info.mint {
            return Err(GauntletError::WrongTokenAccount.into());
        }

        if *gauntlet_state_account.key != vault_info.gauntlet_state_account {
            return Err(GauntletError::WrongVaultStateAccount.into());
        }

        if *vault_state_account.key != vault_strategy_info.vault_account {
            return Err(GauntletError::WrongVaultStrategyStateAccount.into());
        }

        if *gauntlet_state_account.key != strategy_info.gauntlet_state_account {
            return Err(GauntletError::WrongStrategyStateAccount.into());
        }

        if vault_info.deposit_token_account != *vault_deposit_token_account.key {
            return Err(GauntletError::WrongTokenAccount.into());
        }

        if vault_info.reward_token_account != *vault_reward_token_account.key {
            return Err(GauntletError::WrongTokenAccount.into());
        }

        if vault_reward_b_token_account.is_some() {
            if vault_info.reward_token_b_account != *vault_reward_b_token_account.unwrap().key {
                return Err(GauntletError::WrongTokenAccount.into());
            }
        }

        if !vault_strategy_info.availabilities[strategy_index] {
            // 활성화된 strategy가 아닙니다
            return Err(GauntletError::InvalidStatusStrategy.into());
        }

        if depositor_user_info.user_status != 4 {
            return Err(GauntletError::UserStatusError.into());
        }

        let clock = &Clock::get()?;
        if clock.unix_timestamp > depositor_user_info.deadline {
            return Err(GauntletError::TimeoutError.into());
        }

        if depositor_user_info.amount > 0 {
            let user_amount = depositor_user_info.amount as u128;
            let p = (user_amount
                .checked_mul(vault_info.accumulated_reward_per_shares[strategy_index])
                .unwrap()
                .checked_shr(64)
                .unwrap() as u64)
                .checked_sub(depositor_user_info.reward_debt)
                .unwrap();
            depositor_user_info.reward = depositor_user_info.reward.checked_add(p).unwrap();
        }

        if amount > 0 {
            transfer_token(
                &spl_token::id(),
                depositor_deposit_token_account,
                vault_deposit_token_account,
                depositor,
                amount,
            )?;
            match deposit_type {
                DepositType::RAYDIUM => Raydium::raydium_deposit(deposit_accounts, amount).unwrap(),
                DepositType::RAYDIUM_V4 => {
                    Raydium::raydium_deposit_v4(deposit_accounts, amount).unwrap()
                }
            }
            depositor_user_info.amount = depositor_user_info.amount.checked_add(amount).unwrap();
            vault_info.total_deposit_amount =
                vault_info.total_deposit_amount.checked_add(amount).unwrap();
            vault_info.deposit_amounts[strategy_index] = vault_info.deposit_amounts[strategy_index]
                .checked_add(amount)
                .unwrap();
        }

        let user_amount = depositor_user_info.amount as u128;
        depositor_user_info.reward_debt = user_amount
            .checked_mul(vault_info.accumulated_reward_per_shares[strategy_index])
            .unwrap()
            .checked_shr(64)
            .unwrap() as u64;

        depositor_user_info.user_status = 0;
        Vault::pack(vault_info, &mut vault_state_account.data.borrow_mut())?;
        User::pack(
            depositor_user_info,
            &mut depositor_user_state_account.data.borrow_mut(),
        )?;
        Ok(())
    }

    fn withdraw(
        accounts: &[AccountInfo],
        amount: u64,
        mut reward_amount: u64,
        withdraw_type: WithdrawType,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let withdrawer = next_account_info(account_info_iter)?;
        let withdrawer_user_state_account = next_account_info(account_info_iter)?;
        let withdrawer_deposit_token_account = next_account_info(account_info_iter)?;
        let withdrawer_reward_token_account = next_account_info(account_info_iter)?;
        let gauntlet_state_account = next_account_info(account_info_iter)?;
        let vault_state_account = next_account_info(account_info_iter)?;
        let vault_strategy_state_account = next_account_info(account_info_iter)?;
        let strategy_state_account = next_account_info(account_info_iter)?;
        let strategy_token_account = next_account_info(account_info_iter)?;
        let withdraw_fee_token_account = next_account_info(account_info_iter)?;
        let performance_fee_token_account = next_account_info(account_info_iter)?;
        let withdraw_accounts = match withdraw_type {
            WithdrawType::RAYDIUM => next_account_infos(account_info_iter, 11).unwrap(),
            WithdrawType::RAYDIUM_V4 => next_account_infos(account_info_iter, 13).unwrap(),
        };
        let gauntlet_signer_account = &withdraw_accounts[4];
        let vault_deposit_token_account = &withdraw_accounts[5];

        let mut withdrawer_user_info = User::unpack(&withdrawer_user_state_account.data.borrow())?;
        let withdrawer_deposit_token_account_info =
            Account::unpack(&withdrawer_deposit_token_account.data.borrow())?;
        let withdrawer_reward_token_account_info =
            Account::unpack(&withdrawer_reward_token_account.data.borrow())?;
        let mut vault_info = Vault::unpack(&vault_state_account.data.borrow())?;
        let vault_deposit_token_account_info =
            Account::unpack(&vault_deposit_token_account.data.borrow())?;
        let mut vault_strategy_info =
            VaultStrategy::unpack(&vault_strategy_state_account.data.borrow())?;
        let mut strategy_info = Strategy::unpack(&strategy_state_account.data.borrow())?;
        let strategy_token_account_info = Account::unpack(&strategy_token_account.data.borrow())?;

        let vault_index = vault_info.index as usize;
        let strategy_index = strategy_info.index as usize;

        if !withdrawer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if *withdrawer.key != withdrawer_user_info.user {
            return Err(GauntletError::WrongUserAccount.into());
        }

        if withdrawer_user_info.vault_account != *vault_state_account.key {
            return Err(GauntletError::WrongVaultStateAccount.into());
        }

        if *strategy_state_account.key != withdrawer_user_info.strategy_account {
            return Err(GauntletError::WrongUserAccount.into());
        }

        if withdrawer_deposit_token_account_info.mint != vault_deposit_token_account_info.mint {
            return Err(GauntletError::WrongTokenAccount.into());
        }

        if withdrawer_reward_token_account_info.mint != strategy_token_account_info.mint {
            return Err(GauntletError::WrongTokenAccount.into());
        }

        if *gauntlet_state_account.key != vault_info.gauntlet_state_account {
            return Err(GauntletError::WrongVaultStateAccount.into());
        }

        if *vault_state_account.key != vault_strategy_info.vault_account {
            return Err(GauntletError::WrongVaultStrategyStateAccount.into());
        }

        if *gauntlet_state_account.key != strategy_info.gauntlet_state_account {
            return Err(GauntletError::WrongStrategyStateAccount.into());
        }

        if vault_info.deposit_token_account != *vault_deposit_token_account.key {
            return Err(GauntletError::WrongTokenAccount.into());
        }
        if strategy_info.strategy_token_account != *strategy_token_account.key {
            return Err(GauntletError::WrongTokenAccount.into());
        }
        if vault_info.withdraw_fee_account != *withdraw_fee_token_account.key {
            return Err(GauntletError::WrongFeeAccount.into());
        }

        if strategy_info.performance_fee_account != *performance_fee_token_account.key {
            return Err(GauntletError::WrongFeeAccount.into());
        }

        if withdrawer_user_info.user_status != 4 {
            return Err(GauntletError::UserStatusError.into());
        }
        let clock = &Clock::get()?;
        if clock.unix_timestamp > withdrawer_user_info.deadline {
            return Err(GauntletError::TimeoutError.into());
        }
        // 이거 반대 아닐까..!?
        if withdrawer_user_info.amount.lt(&amount) {
            return Err(GauntletError::InvalidWithdrawAmount.into());
        }

        if withdrawer_user_info.amount.gt(&0) {
            let user_amount = withdrawer_user_info.amount as u128;
            let p = (user_amount
                .checked_mul(vault_info.accumulated_reward_per_shares[strategy_index])
                .unwrap()
                .checked_shr(64)
                .unwrap() as u64)
                .checked_sub(withdrawer_user_info.reward_debt)
                .unwrap();
            withdrawer_user_info.reward = withdrawer_user_info.reward.checked_add(p).unwrap();
        }

        if withdrawer_user_info.reward.lt(&reward_amount) {
            return Err(GauntletError::InvalidWithdrawAmount.into());
        }

        if reward_amount.gt(&0) {
            let strat_amount = strategy_info.deposit_amounts[vault_index] as u128;
            reward_amount = withdrawer_user_info.reward;
            let withdraw_amount = strat_amount
                .checked_mul(reward_amount as u128)
                .unwrap()
                .checked_div(vault_strategy_info.strategy_token_amounts[strategy_index] as u128)
                .unwrap() as u64;
            strategy_info.deposit_amounts[vault_index] = strategy_info.deposit_amounts[vault_index]
                .checked_sub(reward_amount)
                .unwrap();
            withdrawer_user_info.reward = withdrawer_user_info
                .reward
                .checked_sub(reward_amount)
                .unwrap();
            vault_strategy_info.strategy_token_amounts[strategy_index] = vault_strategy_info
                .strategy_token_amounts[strategy_index]
                .checked_sub(reward_amount)
                .unwrap();
            let fee = (withdraw_amount as u128)
                .checked_mul(vault_info.fees.performance_fee_numerator as u128)
                .unwrap()
                .checked_div(vault_info.fees.performance_fee_denominator as u128)
                .unwrap() as u64;
            if fee.gt(&0) {
                transfer_token_signed(
                    &spl_token::id(),
                    strategy_token_account,
                    performance_fee_token_account,
                    gauntlet_signer_account,
                    fee,
                )?;
            }
            transfer_token_signed(
                &spl_token::id(),
                strategy_token_account,
                withdrawer_reward_token_account,
                gauntlet_signer_account,
                withdraw_amount.checked_sub(fee).unwrap(),
            )?;
        }

        if amount.gt(&0) {
            match withdraw_type {
                WithdrawType::RAYDIUM => {
                    Raydium::raydium_withdraw(withdraw_accounts, amount).unwrap()
                }
                WithdrawType::RAYDIUM_V4 => {
                    Raydium::raydium_withdraw_v4(withdraw_accounts, amount).unwrap()
                }
            }
            withdrawer_user_info.amount = withdrawer_user_info.amount.checked_sub(amount).unwrap();
            vault_info.deposit_amounts[strategy_index] = vault_info.deposit_amounts[strategy_index]
                .checked_sub(amount)
                .unwrap();
            vault_info.total_deposit_amount =
                vault_info.total_deposit_amount.checked_sub(amount).unwrap();
            let fee = (amount as u128)
                .checked_mul(vault_info.fees.withdrawal_fee_numerator as u128)
                .unwrap()
                .checked_div(vault_info.fees.withdrawal_fee_denominator as u128)
                .unwrap() as u64;
            if fee.gt(&0) {
                transfer_token_signed(
                    &spl_token::id(),
                    vault_deposit_token_account,
                    withdraw_fee_token_account,
                    gauntlet_signer_account,
                    fee,
                )?;
            }
            transfer_token_signed(
                &spl_token::id(),
                vault_deposit_token_account,
                withdrawer_deposit_token_account,
                gauntlet_signer_account,
                amount.checked_sub(fee).unwrap(),
            )?;
        }
        withdrawer_user_info.reward_debt = (withdrawer_user_info.amount as u128)
            .checked_mul(vault_info.accumulated_reward_per_shares[strategy_index])
            .unwrap()
            .checked_shr(64)
            .unwrap() as u64;
        withdrawer_user_info.user_status = 0;
        Vault::pack(vault_info, &mut vault_state_account.data.borrow_mut())?;
        VaultStrategy::pack(
            vault_strategy_info,
            &mut vault_strategy_state_account.data.borrow_mut(),
        )?;
        Strategy::pack(strategy_info, &mut strategy_state_account.data.borrow_mut())?;
        User::pack(
            withdrawer_user_info,
            &mut withdrawer_user_state_account.data.borrow_mut(),
        )?;

        Ok(())
    }

    fn _harvest(
        gauntlet_account_info: &Gauntlet,
        vault_account_info: &mut Vault,
        vault_strategy_info: &VaultStrategy,
        harvest_accounts: &[AccountInfo],
        vault_reward_token_account: &AccountInfo,
        vault_reward_b_token_account: &Option<&AccountInfo>,
        deposit_type: &DepositType,
    ) -> ProgramResult {
        // _harvest함수는 farm_reward_token, farm_reward_token_b를 raydium에서 harvest한후 vault_state에 각 strategies에 배분될 farm_reward_token들 양을 계산해서 업데이트까지만 함
        let vault_reward_token_account_info =
            Account::unpack(&vault_reward_token_account.data.borrow())?;
        let before_reward_token_balance = vault_reward_token_account_info.amount;
        let strategies_len = gauntlet_account_info.strategies_len as usize;
        if vault_reward_b_token_account.is_some() {
            let vault_reward_b_token_account = vault_reward_b_token_account.unwrap();
            let vault_reward_b_token_account_info =
                Account::unpack(&vault_reward_b_token_account.data.borrow())?;
            let before_reward_b_token_balance = vault_reward_b_token_account_info.amount;

            match deposit_type {
                DepositType::RAYDIUM => Raydium::raydium_harvest(harvest_accounts).unwrap(),
                DepositType::RAYDIUM_V4 => Raydium::raydium_harvest_v4(harvest_accounts).unwrap(),
            }
            let vault_reward_token_account_info =
                Account::unpack(&vault_reward_token_account.data.borrow())?;
            let vault_reward_b_token_account_info =
                Account::unpack(&vault_reward_b_token_account.data.borrow())?;

            // reward token harvest 된 양 계산
            let reward_token_harvest_amount = vault_reward_token_account_info
                .amount
                .checked_sub(before_reward_token_balance)
                .unwrap() as u128;

            let reward_b_token_harvest_amount = vault_reward_b_token_account_info
                .amount
                .checked_sub(before_reward_b_token_balance)
                .unwrap() as u128;

            // 각 strategies에 deposit token양 비율 만큼 reward token양 배분
            for i in 0..strategies_len {
                if vault_strategy_info.availabilities[i] {
                    // availabilty가 true일때만 reward 계산 및 분배
                    vault_account_info.reward_token_remain_amounts[i] = vault_account_info
                        .reward_token_remain_amounts[i]
                        .checked_add(
                            reward_token_harvest_amount
                                .checked_mul(vault_account_info.deposit_amounts[i] as u128)
                                .unwrap()
                                .checked_div(vault_account_info.total_deposit_amount as u128)
                                .unwrap() as u64,
                        )
                        .unwrap();

                    vault_account_info.reward_token_b_remain_amounts[i] = vault_account_info
                        .reward_token_b_remain_amounts[i]
                        .checked_add(
                            reward_b_token_harvest_amount
                                .checked_mul(vault_account_info.deposit_amounts[i] as u128)
                                .unwrap()
                                .checked_div(vault_account_info.total_deposit_amount as u128)
                                .unwrap() as u64,
                        )
                        .unwrap();
                }
            }
        } else {
            match deposit_type {
                DepositType::RAYDIUM => Raydium::raydium_harvest(harvest_accounts).unwrap(),
                DepositType::RAYDIUM_V4 => Raydium::raydium_harvest_v4(harvest_accounts).unwrap(),
            }
            let vault_reward_token_account_info =
                Account::unpack(&vault_reward_token_account.data.borrow())?;
            let reward_token_harvest_amount = vault_reward_token_account_info
                .amount
                .checked_sub(before_reward_token_balance)
                .unwrap() as u128;
            // 각 Strategy별 swap하기를 나기다리는 남은 reward의 양을 업데이트함
            for i in 0..strategies_len {
                if vault_strategy_info.availabilities[i] {
                    // availabilty가 true일때만 reward 계산 및 분배
                    vault_account_info.reward_token_remain_amounts[i] = vault_account_info
                        .reward_token_remain_amounts[i]
                        .checked_add(
                            reward_token_harvest_amount
                                .checked_mul(vault_account_info.deposit_amounts[i] as u128)
                                .unwrap()
                                .checked_div(vault_account_info.total_deposit_amount as u128)
                                .unwrap() as u64,
                        )
                        .unwrap();
                }
            }
        }
        Ok(())
    }

    fn _swap_farm_token_to_usdc(
        vault_account_info: &mut Vault,
        strategy_index: usize,
        usdc_token_account: &AccountInfo,
        swap_reward_to_usdc_accounts: &[AccountInfo],
        swap_type: &SwapType,
        second_reward_token: bool,
    ) -> ProgramResult {
        let usdc_token_account_info = Account::unpack(&usdc_token_account.data.borrow())?;
        let before_usdc_token_amount = usdc_token_account_info.amount;
        let reward_token_remain_amounts = match second_reward_token {
            false => vault_account_info.reward_token_remain_amounts[strategy_index],
            true => vault_account_info.reward_token_b_remain_amounts[strategy_index],
        };
        if reward_token_remain_amounts.gt(&0) {
            match swap_type {
                SwapType::RAYDIUM => {
                    Self::raydium_swap(
                        swap_reward_to_usdc_accounts,
                        reward_token_remain_amounts,
                        0,
                    )
                    .unwrap();
                }
            }
            match second_reward_token {
                false => vault_account_info.reward_token_remain_amounts[strategy_index] = 0,
                true => vault_account_info.reward_token_b_remain_amounts[strategy_index] = 0,
            }
            let usdc_token_account_info = Account::unpack(&usdc_token_account.data.borrow())?;
            let swap_amount = usdc_token_account_info
                .amount
                .checked_sub(before_usdc_token_amount)
                .unwrap() as u128;

            vault_account_info.usdc_token_amounts[strategy_index] = vault_account_info
                .usdc_token_amounts[strategy_index]
                .checked_add(swap_amount as u64)
                .unwrap(); // 스왑한 usdc amount를 vault state에 update
        }

        Ok(())
    }

    fn _swap_usdc_to_strategy_token(
        vault_account_info: &mut Vault,
        vault_strategy_account_info: &mut VaultStrategy,
        strategy_account_info: &mut Strategy,
        strategy_token_account: &AccountInfo,
        usdc_token_account: &AccountInfo,
        swap_usdc_to_strategy_accounts: &[AccountInfo],
        swap_type: &SwapType,
    ) -> ProgramResult {
        let vault_index = vault_account_info.index as usize;
        let strategy_index = strategy_account_info.index as usize;

        let available_usdc_amount = vault_account_info.usdc_token_amounts[strategy_index];

        let usdc_token_account_info = Account::unpack(&usdc_token_account.data.borrow())?;

        let before_usdc_balance = usdc_token_account_info.amount;

        let strategy_token_account_info = Account::unpack(&strategy_token_account.data.borrow())?;
        let before_strategy_token_amount = strategy_token_account_info.amount;
        if available_usdc_amount.gt(&0) {
            match swap_type {
                SwapType::RAYDIUM => {
                    Processor::raydium_swap(
                        swap_usdc_to_strategy_accounts,
                        available_usdc_amount,
                        0,
                    )
                    .unwrap();
                }
            }

            let usdc_token_account_info = Account::unpack(&usdc_token_account.data.borrow())?;
            let swaped_usdc_amount = before_usdc_balance
                .checked_sub(usdc_token_account_info.amount)
                .unwrap();
            vault_account_info.usdc_token_amounts[strategy_index] = available_usdc_amount
                .checked_sub(swaped_usdc_amount)
                .unwrap(); // swap하고 남은 짜투리 usdc양 업데이트

            let strategy_token_account_info =
                Account::unpack(&strategy_token_account.data.borrow())?;
            let swap_amount = strategy_token_account_info
                .amount
                .checked_sub(before_strategy_token_amount)
                .unwrap() as u128;
            // 해당 strategy의 acc 업데이트
            vault_account_info.accumulated_reward_per_shares[strategy_index] = vault_account_info
                .accumulated_reward_per_shares[strategy_index]
                .checked_add(
                    swap_amount
                        .checked_shl(64)
                        .unwrap()
                        .checked_div(vault_account_info.deposit_amounts[strategy_index] as u128)
                        .unwrap(),
                )
                .unwrap();

            // 해당 strategy state들 업데이트
            strategy_account_info.total_deposit_amount = strategy_account_info
                .total_deposit_amount
                .checked_add(swap_amount as u64)
                .unwrap();
            strategy_account_info.deposit_amounts[vault_index] = strategy_account_info
                .deposit_amounts[vault_index]
                .checked_add(swap_amount as u64)
                .unwrap();

            vault_strategy_account_info.strategy_token_amounts[strategy_index] =
                vault_strategy_account_info.strategy_token_amounts[strategy_index]
                    .checked_add(swap_amount as u64)
                    .unwrap();
        }
        Ok(())
    }

    fn _swap_reward_to_strategy_token(
        vault_account_info: &mut Vault,
        vault_strategy_account_info: &mut VaultStrategy,
        strategy_account_info: &mut Strategy,
        strategy_token_account: &AccountInfo,
        swap_reward_to_strategy_accounts: &[AccountInfo],
        swap_type: &SwapType,
        second_reward_token: bool,
    ) -> ProgramResult {
        let vault_index = vault_account_info.index as usize;
        let strategy_index = strategy_account_info.index as usize;

        let reward_token_remain_amounts = match second_reward_token {
            false => vault_account_info.reward_token_remain_amounts[strategy_index],
            true => vault_account_info.reward_token_b_remain_amounts[strategy_index],
        };
        let strategy_token_account_info = Account::unpack(&strategy_token_account.data.borrow())?;
        let before_strategy_token_amount = strategy_token_account_info.amount;
        if reward_token_remain_amounts.gt(&0) {
            match swap_type {
                SwapType::RAYDIUM => {
                    Processor::raydium_swap(
                        swap_reward_to_strategy_accounts,
                        reward_token_remain_amounts,
                        0,
                    )
                    .unwrap();
                }
            }
            match second_reward_token {
                false => vault_account_info.reward_token_remain_amounts[strategy_index] = 0,
                true => vault_account_info.reward_token_b_remain_amounts[strategy_index] = 0,
            }

            let strategy_token_account_info =
                Account::unpack(&strategy_token_account.data.borrow())?;
            let swap_amount = strategy_token_account_info
                .amount
                .checked_sub(before_strategy_token_amount)
                .unwrap() as u128;
            // 해당 strategy의 acc 업데이트
            vault_account_info.accumulated_reward_per_shares[strategy_index] = vault_account_info
                .accumulated_reward_per_shares[strategy_index]
                .checked_add(
                    swap_amount
                        .checked_shl(64)
                        .unwrap()
                        .checked_div(vault_account_info.deposit_amounts[strategy_index] as u128)
                        .unwrap(),
                )
                .unwrap();

            // 해당 strategy state들 업데이트
            strategy_account_info.total_deposit_amount = strategy_account_info
                .total_deposit_amount
                .checked_add(swap_amount as u64)
                .unwrap();
            strategy_account_info.deposit_amounts[vault_index] = strategy_account_info
                .deposit_amounts[vault_index]
                .checked_add(swap_amount as u64)
                .unwrap();

            vault_strategy_account_info.strategy_token_amounts[strategy_index] =
                vault_strategy_account_info.strategy_token_amounts[strategy_index]
                    .checked_add(swap_amount as u64)
                    .unwrap();
        }
        Ok(())
    }

    fn create_user_account(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let depositor = next_account_info(account_info_iter)?;
        let vault_state_account = next_account_info(account_info_iter)?;
        let strategy_state_account = next_account_info(account_info_iter)?;
        let depositor_user_state_account = next_account_info(account_info_iter)?;
        let system_program_account = next_account_info(account_info_iter)?;
        let (_pda, _seed) = Pubkey::find_program_address(
            &[
                &vault_state_account.key.to_bytes(),
                &depositor.key.to_bytes(),
                &strategy_state_account.key.to_bytes(),
            ],
            program_id,
        );
        if *depositor_user_state_account.key != _pda {
            return Err(ProgramError::InvalidSeeds);
        }
        create_pda_account(
            depositor,
            130,
            program_id,
            system_program_account,
            depositor_user_state_account,
            &[
                &vault_state_account.key.to_bytes(),
                &depositor.key.to_bytes(),
                &strategy_state_account.key.to_bytes(),
                &[_seed],
            ],
        )?;
        Ok(())
    }
}
