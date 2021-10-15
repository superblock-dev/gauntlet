use crate::error::GauntletError;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    clock::{Clock, UnixTimestamp},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use std::convert::TryFrom;

#[derive(PartialEq, Clone, Copy)]
pub enum Status {
    PAUSED,
    NORMAL,
}

impl Default for Status {
    fn default() -> Self {
        Status::NORMAL
    }
}

/// Encapsulates all fee information and calculations for swap operations
#[derive(Debug)]
pub struct Fees {
    /// Performance fee numerator
    pub performance_fee_numerator: u64,
    /// Performance fee denominator
    pub performance_fee_denominator: u64,
    /// Withdrawal fee numerator
    pub withdrawal_fee_numerator: u64,
    /// Withdrawal fee denominator
    pub withdrawal_fee_denominator: u64,
}

/// Helper function for calculating fee
pub fn calculate_fee(
    token_amount: u128,
    fee_numerator: u128,
    fee_denominator: u128,
) -> Option<u128> {
    if fee_numerator == 0 || token_amount == 0 {
        Some(0)
    } else {
        let fee = token_amount
            .checked_mul(fee_numerator)?
            .checked_div(fee_denominator)?;
        if fee == 0 {
            Some(1) // minimum fee of one token
        } else {
            Some(fee)
        }
    }
}

fn validate_fraction(numerator: u64, denominator: u64) -> Result<(), GauntletError> {
    if denominator == 0 && numerator == 0 {
        Ok(())
    } else if numerator >= denominator {
        Err(GauntletError::InvalidFee)
    } else {
        Ok(())
    }
}

impl Fees {
    /// Calculate the performance fee in pool tokens
    pub fn performance_fee(&self, reward_tokens: u128) -> Option<u128> {
        calculate_fee(
            reward_tokens,
            u128::try_from(self.performance_fee_numerator).ok()?,
            u128::try_from(self.performance_fee_denominator).ok()?,
        )
    }

    /// Calculate the withdraw fee in pool tokens
    pub fn withdrawal_fee(&self, pool_tokens: u128) -> Option<u128> {
        calculate_fee(
            pool_tokens,
            u128::try_from(self.withdrawal_fee_numerator).ok()?,
            u128::try_from(self.withdrawal_fee_denominator).ok()?,
        )
    }

    /// Validate that the fees are reasonable
    pub fn validate(&self) -> Result<(), GauntletError> {
        validate_fraction(
            self.performance_fee_numerator,
            self.performance_fee_denominator,
        )?;
        validate_fraction(
            self.withdrawal_fee_numerator,
            self.withdrawal_fee_denominator,
        )?;

        Ok(())
    }
}

impl Sealed for Fees {}
impl IsInitialized for Fees {
    fn is_initialized(&self) -> bool {
        true
    }
}

impl Pack for Fees {
    const LEN: usize = 32;

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, 32];
        let (
            performance_fee_numerator,
            performance_fee_denominator,
            withdrawal_fee_numerator,
            withdrawal_fee_denominator,
        ) = mut_array_refs![output, 8, 8, 8, 8];
        *performance_fee_numerator = self.performance_fee_numerator.to_le_bytes();
        *performance_fee_denominator = self.performance_fee_denominator.to_le_bytes();
        *withdrawal_fee_numerator = self.withdrawal_fee_numerator.to_le_bytes();
        *withdrawal_fee_denominator = self.withdrawal_fee_denominator.to_le_bytes();
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Fees, ProgramError> {
        let input = array_ref![input, 0, 32];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            performance_fee_numerator,
            performance_fee_denominator,
            withdrawal_fee_numerator,
            withdrawal_fee_denominator,
        ) = array_refs![input, 8, 8, 8, 8];
        Ok(Self {
            performance_fee_numerator: u64::from_le_bytes(*performance_fee_numerator),
            performance_fee_denominator: u64::from_le_bytes(*performance_fee_denominator),
            withdrawal_fee_numerator: u64::from_le_bytes(*withdrawal_fee_numerator),
            withdrawal_fee_denominator: u64::from_le_bytes(*withdrawal_fee_denominator),
        })
    }
}
pub struct Gauntlet {
    /// init
    pub is_initialized: bool,
    /// admin account
    pub admin: Pubkey,
    /// Number of strategies,
    pub strategies_len: u8,
    /// Number of strategies,
    pub vaults_len: u8,
    /// usdc token account for swap
    pub usdc_token_account: Pubkey,
}

impl Gauntlet {
    pub fn init(admin: Pubkey, usdc_token_account: Pubkey) -> Self {
        Gauntlet {
            is_initialized: true,
            admin,
            strategies_len: 0,
            vaults_len: 0,
            usdc_token_account,
        }
    }
}

impl Sealed for Gauntlet {}
impl IsInitialized for Gauntlet {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Gauntlet {
    const LEN: usize = 1 + 32 + 8 + 8 + 32; // 81
    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, Gauntlet::LEN];
        let (is_initialized, admin, strategies_len, vaults_len, usdc_token_account) =
            mut_array_refs![output, 1, 32, 8, 8, 32];

        is_initialized[0] = self.is_initialized as u8;
        admin.copy_from_slice(self.admin.as_ref());
        strategies_len[0] = self.strategies_len as u8;
        vaults_len[0] = self.vaults_len as u8;
        usdc_token_account.copy_from_slice(self.usdc_token_account.as_ref());
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, Gauntlet::LEN];
        let (is_initialized, admin, strategies_len, vaults_len, usdc_token_account) =
            array_refs![input, 1, 32, 8, 8, 32];

        Ok(Self {
            is_initialized: match is_initialized {
                [0] => false,
                [1] => true,
                _ => return Err(ProgramError::InvalidAccountData),
            },
            admin: Pubkey::new_from_array(*admin),
            strategies_len: strategies_len[0],
            vaults_len: vaults_len[0],
            usdc_token_account: Pubkey::new_from_array(*usdc_token_account),
        })
    }
}
pub struct User {
    /// Initialized state
    pub is_initialized: bool,
    /// user pubkey
    pub user: Pubkey,
    /// vault
    pub vault_account: Pubkey,
    /// strategy
    pub strategy_account: Pubkey,
    // User deposit lp amount
    pub amount: u64,
    // Withdrawable reward amount
    pub reward: u64,
    // Value for calculate user's pending reward amount
    pub reward_debt: u64,
    // user status
    pub user_status: u8,
    // last timestamp
    pub deadline: UnixTimestamp,
}

impl User {
    pub fn init(user: Pubkey, vault_account: Pubkey, strategy_account: Pubkey) -> Self {
        User {
            is_initialized: true,
            user,
            vault_account,
            strategy_account,
            amount: 0,
            reward: 0,
            reward_debt: 0,
            user_status: 0,
            deadline: 0,
        }
    }
}

impl Sealed for User {}
impl IsInitialized for User {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for User {
    const LEN: usize = 130;
    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, User::LEN];
        let (
            is_initialized,
            user,
            vault_account,
            strategy_account,
            amount,
            reward,
            reward_debt,
            user_status,
            deadline,
        ) = mut_array_refs![output, 1, 32, 32, 32, 8, 8, 8, 1, 8];

        is_initialized[0] = self.is_initialized as u8;
        user.copy_from_slice(self.user.as_ref());
        vault_account.copy_from_slice(self.vault_account.as_ref());
        strategy_account.copy_from_slice(self.strategy_account.as_ref());
        *amount = self.amount.to_le_bytes();
        *reward = self.reward.to_le_bytes();
        *reward_debt = self.reward_debt.to_le_bytes();
        user_status[0] = self.user_status as u8;
        *deadline = self.deadline.to_le_bytes();
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, User::LEN];
        let (
            is_initialized,
            user,
            vault_account,
            strategy_account,
            amount,
            reward,
            reward_debt,
            user_status,
            deadline,
        ) = array_refs![input, 1, 32, 32, 32, 8, 8, 8, 1, 8];

        Ok(Self {
            is_initialized: match is_initialized {
                [0] => false,
                [1] => true,
                _ => return Err(ProgramError::InvalidAccountData),
            },
            user: Pubkey::new_from_array(*user),
            vault_account: Pubkey::new_from_array(*vault_account),
            strategy_account: Pubkey::new_from_array(*strategy_account),
            amount: u64::from_le_bytes(*amount),
            reward: u64::from_le_bytes(*reward),
            reward_debt: u64::from_le_bytes(*reward_debt),
            user_status: user_status[0],
            deadline: UnixTimestamp::from_le_bytes(*deadline),
        })
    }
}

/// 전략 개수 상한 : 일단 50개로 잡아놓음 * TODO
pub const MAX_NUMBER_OF_STRATEGY: usize = 50;
pub const MAX_VAULT_SIZE: usize = 1
    + 1
    + 1
    + Fees::LEN
    + 32
    + 32
    + 32
    + 32
    + 32
    + 8
    + 8 * 4 * MAX_NUMBER_OF_STRATEGY
    + 16 * MAX_NUMBER_OF_STRATEGY
    + 8
    + 32;
pub struct Vault {
    /// Initialized state
    pub is_initialized: bool,
    /// Vault index
    pub index: u8,
    /// Vault's running status
    pub status: Status,
    /// Vault fees
    pub fees: Fees,
    /// Gauntlet Account,
    pub gauntlet_state_account: Pubkey,
    /// Deposit token(LP) account address
    pub deposit_token_account: Pubkey,
    /// farm reward token account
    pub reward_token_account: Pubkey,
    /// farm reward token b account
    pub reward_token_b_account: Pubkey,
    /// withdraw fee account
    pub withdraw_fee_account: Pubkey,
    /// Total deposit token amount
    pub total_deposit_amount: u64,
    /// Deposit token amount
    pub deposit_amounts: Vec<u64>,
    /// Total remain token amount,
    pub reward_token_remain_amounts: Vec<u64>,
    /// Total remain token amount,
    pub reward_token_b_remain_amounts: Vec<u64>,
    /// usdc token amount,
    pub usdc_token_amounts: Vec<u64>,
    /// Accumulated reward per share
    pub accumulated_reward_per_shares: Vec<u128>,
    /// Last reward update time
    pub last_reward_update_time: UnixTimestamp,
    /// raydium state account
    pub raydium_state_account: Pubkey,
}

impl Sealed for Vault {}

impl IsInitialized for Vault {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Vault {
    const LEN: usize = MAX_VAULT_SIZE; // 2251

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, Vault::LEN];
        let (
            is_initialized,
            index,
            status,
            fees,
            gauntlet_state_account,
            deposit_token_account,
            reward_token_account,
            reward_token_b_account,
            withdraw_fee_account,
            total_deposit_amount,
            deposit_amounts,
            reward_token_remain_amounts,
            reward_token_b_remain_amounts,
            usdc_token_amounts,
            accumulated_reward_per_shares,
            last_reward_update_time,
            raydium_state_account,
        ) = mut_array_refs![
            output,
            1,
            1,
            1,
            Fees::LEN,
            32,
            32,
            32,
            32,
            32,
            8,
            8 * MAX_NUMBER_OF_STRATEGY,
            8 * MAX_NUMBER_OF_STRATEGY,
            8 * MAX_NUMBER_OF_STRATEGY,
            8 * MAX_NUMBER_OF_STRATEGY,
            16 * MAX_NUMBER_OF_STRATEGY,
            8,
            32
        ];
        is_initialized[0] = self.is_initialized as u8;
        index[0] = self.index as u8;
        status[0] = self.status as u8;
        self.fees.pack_into_slice(&mut fees[..]);
        gauntlet_state_account.copy_from_slice(self.gauntlet_state_account.as_ref());
        deposit_token_account.copy_from_slice(self.deposit_token_account.as_ref());
        reward_token_account.copy_from_slice(self.reward_token_account.as_ref());
        reward_token_b_account.copy_from_slice(self.reward_token_b_account.as_ref());
        withdraw_fee_account.copy_from_slice(self.withdraw_fee_account.as_ref());
        *total_deposit_amount = self.total_deposit_amount.to_le_bytes();
        for i in 0..MAX_NUMBER_OF_STRATEGY {
            let arr_ref = array_mut_ref![deposit_amounts, i * 8, 8];
            *arr_ref = self.deposit_amounts[i].to_le_bytes();
        }
        for i in 0..MAX_NUMBER_OF_STRATEGY {
            let arr_ref = array_mut_ref![reward_token_remain_amounts, i * 8, 8];
            *arr_ref = self.reward_token_remain_amounts[i].to_le_bytes();
        }
        for i in 0..MAX_NUMBER_OF_STRATEGY {
            let arr_ref = array_mut_ref![reward_token_b_remain_amounts, i * 8, 8];
            *arr_ref = self.reward_token_b_remain_amounts[i].to_le_bytes();
        }
        for i in 0..MAX_NUMBER_OF_STRATEGY {
            let arr_ref = array_mut_ref![usdc_token_amounts, i * 8, 8];
            *arr_ref = self.usdc_token_amounts[i].to_le_bytes();
        }
        for i in 0..MAX_NUMBER_OF_STRATEGY {
            let arr_ref = array_mut_ref![accumulated_reward_per_shares, i * 16, 16];
            *arr_ref = self.accumulated_reward_per_shares[i].to_le_bytes();
        }
        *last_reward_update_time = self.last_reward_update_time.to_le_bytes();
        raydium_state_account.copy_from_slice(self.raydium_state_account.as_ref());
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, Vault::LEN];
        let (
            is_initialized,
            index,
            status,
            fees,
            gauntlet_state_account,
            deposit_token_account,
            reward_token_account,
            reward_token_b_account,
            withdraw_fee_account,
            total_deposit_amount,
            deposit_amounts,
            reward_token_remain_amounts,
            reward_token_b_remain_amounts,
            usdc_token_amounts,
            accumulated_reward_per_shares,
            last_reward_update_time,
            raydium_state_account,
        ) = array_refs![
            input,
            1,
            1,
            1,
            Fees::LEN,
            32,
            32,
            32,
            32,
            32,
            8,
            8 * MAX_NUMBER_OF_STRATEGY,
            8 * MAX_NUMBER_OF_STRATEGY,
            8 * MAX_NUMBER_OF_STRATEGY,
            8 * MAX_NUMBER_OF_STRATEGY,
            16 * MAX_NUMBER_OF_STRATEGY,
            8,
            32
        ];
        let mut deposit_amounts_array = vec![0; MAX_NUMBER_OF_STRATEGY];
        for i in 0..MAX_NUMBER_OF_STRATEGY {
            let arr_ref = array_ref![deposit_amounts, i * 8, 8];
            deposit_amounts_array[i] = u64::from_le_bytes(*arr_ref);
        }
        let mut reward_token_remain_amounts_array = vec![0; MAX_NUMBER_OF_STRATEGY];
        for i in 0..MAX_NUMBER_OF_STRATEGY {
            let arr_ref = array_ref![reward_token_remain_amounts, i * 8, 8];
            reward_token_remain_amounts_array[i] = u64::from_le_bytes(*arr_ref);
        }
        let mut reward_token_b_remain_amounts_array = vec![0; MAX_NUMBER_OF_STRATEGY];
        for i in 0..MAX_NUMBER_OF_STRATEGY {
            let arr_ref = array_ref![reward_token_b_remain_amounts, i * 8, 8];
            reward_token_b_remain_amounts_array[i] = u64::from_le_bytes(*arr_ref);
        }
        let mut usdc_token_amounts_array = vec![0; MAX_NUMBER_OF_STRATEGY];
        for i in 0..MAX_NUMBER_OF_STRATEGY {
            let arr_ref = array_ref![usdc_token_amounts, i * 8, 8];
            usdc_token_amounts_array[i] = u64::from_le_bytes(*arr_ref);
        }
        let mut accumulated_reward_per_shares_array = vec![0; MAX_NUMBER_OF_STRATEGY];
        for i in 0..MAX_NUMBER_OF_STRATEGY {
            let arr_ref = array_ref![accumulated_reward_per_shares, i * 16, 16];
            accumulated_reward_per_shares_array[i] = u128::from_le_bytes(*arr_ref);
        }
        Ok(Vault {
            is_initialized: match is_initialized {
                [0] => false,
                [1] => true,
                _ => return Err(ProgramError::InvalidAccountData),
            },
            index: index[0],
            status: match status {
                [0] => Status::PAUSED,
                [1] => Status::NORMAL,
                _ => return Err(ProgramError::InvalidAccountData),
            },
            fees: Fees::unpack_from_slice(fees)?,
            gauntlet_state_account: Pubkey::new_from_array(*gauntlet_state_account),
            deposit_token_account: Pubkey::new_from_array(*deposit_token_account),
            reward_token_account: Pubkey::new_from_array(*reward_token_account),
            reward_token_b_account: Pubkey::new_from_array(*reward_token_b_account),
            withdraw_fee_account: Pubkey::new_from_array(*withdraw_fee_account),
            total_deposit_amount: u64::from_le_bytes(*total_deposit_amount),
            deposit_amounts: deposit_amounts_array,
            reward_token_remain_amounts: reward_token_remain_amounts_array,
            reward_token_b_remain_amounts: reward_token_b_remain_amounts_array,
            usdc_token_amounts: usdc_token_amounts_array,
            accumulated_reward_per_shares: accumulated_reward_per_shares_array,
            last_reward_update_time: UnixTimestamp::from_le_bytes(*last_reward_update_time),
            raydium_state_account: Pubkey::new_from_array(*raydium_state_account),
        })
    }
}

pub const MAX_NUMBER_OF_VAULTS: usize = 50;
/// 전략 정보
pub struct Strategy {
    /// Initialized state
    pub is_initialized: bool,
    /// Strategy index
    pub index: u8,
    /// Gauntlet Account,
    pub gauntlet_state_account: Pubkey,
    /// Strategy admin
    pub admin: Pubkey,
    /// Performance fee account
    pub performance_fee_account: Pubkey,
    /// Strategy's running status // status가 paused로 변경되면 관련된 vaultStrategy의 availabilty도 false값으로 변경해줘야함!
    pub status: Status,
    /// Last reward update time
    pub last_reward_update_time: UnixTimestamp,
    /// Total deposit token amount,
    pub total_deposit_amount: u64,
    /// Deposit token amount,
    pub deposit_amounts: Vec<u64>,
    /// Strategy Token Account
    pub strategy_token_account: Pubkey,
}
impl Strategy {
    pub fn init(
        index: u8,
        gauntlet_state_account: Pubkey,
        admin: Pubkey,
        performance_fee_account: Pubkey,
        strategy_token_account: Pubkey,
    ) -> Self {
        Strategy {
            is_initialized: true,
            index,
            gauntlet_state_account,
            admin,
            performance_fee_account,
            status: Status::default(),
            last_reward_update_time: Clock::get().unwrap().unix_timestamp,
            total_deposit_amount: 0,
            deposit_amounts: vec![0; MAX_NUMBER_OF_VAULTS],
            strategy_token_account,
        }
    }
}

impl Sealed for Strategy {}
impl IsInitialized for Strategy {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Strategy {
    const LEN: usize = 1 + 1 + 32 + 32 + 32 + 1 + 8 + 8 + 8 * MAX_NUMBER_OF_VAULTS + 32; // 515

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, Strategy::LEN];
        let (
            is_initialized,
            index,
            gauntlet_state_account,
            admin,
            performance_fee_account,
            status,
            last_reward_update_time,
            total_deposit_amount,
            deposit_amounts,
            strategy_token_account,
        ) = mut_array_refs![
            output,
            1,
            1,
            32,
            32,
            32,
            1,
            8,
            8,
            8 * MAX_NUMBER_OF_VAULTS,
            32
        ];

        is_initialized[0] = self.is_initialized as u8;
        index[0] = self.index as u8;
        gauntlet_state_account.copy_from_slice(self.gauntlet_state_account.as_ref());
        admin.copy_from_slice(self.admin.as_ref());
        performance_fee_account.copy_from_slice(self.performance_fee_account.as_ref());
        status[0] = self.status as u8;
        *last_reward_update_time = self.last_reward_update_time.to_le_bytes();
        *total_deposit_amount = self.total_deposit_amount.to_le_bytes();
        for i in 0..MAX_NUMBER_OF_VAULTS {
            let strategy_deposit_amount = array_mut_ref![deposit_amounts, i * 8, 8];
            *strategy_deposit_amount = self.deposit_amounts[i].to_le_bytes();
        }
        strategy_token_account.copy_from_slice(self.strategy_token_account.as_ref());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Strategy::LEN];
        let (
            is_initialized,
            index,
            gauntlet_state_account,
            admin,
            performance_fee_account,
            status,
            last_reward_update_time,
            total_deposit_amount,
            deposit_amounts,
            strategy_token_account,
        ) = array_refs![src, 1, 1, 32, 32, 32, 1, 8, 8, 8 * MAX_NUMBER_OF_VAULTS, 32];
        let mut deposit_amounts_array = vec![0; MAX_NUMBER_OF_VAULTS];

        for i in 0..MAX_NUMBER_OF_VAULTS {
            let strategy_deposit_amount = array_ref![deposit_amounts, i * 8, 8];
            deposit_amounts_array[i] = u64::from_le_bytes(*strategy_deposit_amount);
        }

        Ok(Strategy {
            is_initialized: match is_initialized {
                [0] => false,
                [1] => true,
                _ => return Err(ProgramError::InvalidAccountData),
            },
            index: u8::from_le_bytes(*index),
            gauntlet_state_account: Pubkey::new_from_array(*gauntlet_state_account),
            admin: Pubkey::new_from_array(*admin),
            performance_fee_account: Pubkey::new_from_array(*performance_fee_account),
            status: match status {
                [0] => Status::PAUSED,
                [1] => Status::NORMAL,
                _ => return Err(ProgramError::InvalidAccountData),
            },
            last_reward_update_time: UnixTimestamp::from_le_bytes(*last_reward_update_time),
            total_deposit_amount: u64::from_le_bytes(*total_deposit_amount),
            deposit_amounts: deposit_amounts_array,
            strategy_token_account: Pubkey::new_from_array(*strategy_token_account),
        })
    }
}

pub struct VaultStrategy {
    /// Initialized state
    pub is_initialized: bool,
    /// vault
    pub vault_account: Pubkey,
    /// need usdc pool
    pub needs_usdc_pools: Vec<bool>,
    /// vault and strategy mapping status
    pub availabilities: Vec<bool>,
    // User deposit reward token amount (BTC, ETH 등)
    pub strategy_token_amounts: Vec<u64>,
}
impl VaultStrategy {
    pub fn init(vault_account: Pubkey) -> Self {
        VaultStrategy {
            is_initialized: true,
            vault_account,
            needs_usdc_pools: vec![false; MAX_NUMBER_OF_STRATEGY],
            availabilities: vec![false; MAX_NUMBER_OF_STRATEGY],
            strategy_token_amounts: vec![0; MAX_NUMBER_OF_STRATEGY],
        }
    }
}

impl Sealed for VaultStrategy {}
impl IsInitialized for VaultStrategy {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for VaultStrategy {
    const LEN: usize =
        1 + 32 + 8 * MAX_NUMBER_OF_STRATEGY + MAX_NUMBER_OF_STRATEGY + MAX_NUMBER_OF_STRATEGY;

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, VaultStrategy::LEN];
        let (
            is_initialized,
            vault_account,
            needs_usdc_pools,
            availabilities,
            strategy_token_amounts,
        ) = mut_array_refs![
            output,
            1,
            32,
            MAX_NUMBER_OF_STRATEGY,
            MAX_NUMBER_OF_STRATEGY,
            8 * MAX_NUMBER_OF_STRATEGY
        ];

        is_initialized[0] = self.is_initialized as u8;
        vault_account.copy_from_slice(self.vault_account.as_ref());
        for i in 0..MAX_NUMBER_OF_STRATEGY {
            let arr_ref = array_mut_ref![needs_usdc_pools, i, 1];
            arr_ref[0] = self.needs_usdc_pools[i] as u8;
        }
        for i in 0..MAX_NUMBER_OF_STRATEGY {
            let arr_ref = array_mut_ref![availabilities, i, 1];
            arr_ref[0] = self.availabilities[i] as u8;
        }
        for i in 0..MAX_NUMBER_OF_STRATEGY {
            let arr_ref = array_mut_ref![strategy_token_amounts, i * 8, 8];
            *arr_ref = self.strategy_token_amounts[i].to_le_bytes();
        }
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, VaultStrategy::LEN];
        let (
            is_initialized,
            vault_account,
            needs_usdc_pools,
            availabilities,
            strategy_token_amounts,
        ) = array_refs![
            src,
            1,
            32,
            MAX_NUMBER_OF_STRATEGY,
            MAX_NUMBER_OF_STRATEGY,
            8 * MAX_NUMBER_OF_STRATEGY
        ];

        let mut needs_usdc_pools_array = vec![false; MAX_NUMBER_OF_STRATEGY];
        for i in 0..MAX_NUMBER_OF_STRATEGY {
            let arr_ref = array_ref![needs_usdc_pools, i, 1];
            needs_usdc_pools_array[i] = match arr_ref {
                [0] => false,
                [1] => true,
                _ => return Err(ProgramError::InvalidAccountData),
            }
        }
        let mut availabilities_array = vec![false; MAX_NUMBER_OF_STRATEGY];
        for i in 0..MAX_NUMBER_OF_STRATEGY {
            let arr_ref = array_ref![availabilities, i, 1];
            availabilities_array[i] = match arr_ref {
                [0] => false,
                [1] => true,
                _ => return Err(ProgramError::InvalidAccountData),
            }
        }
        let mut strategy_token_amounts_array = vec![0; MAX_NUMBER_OF_STRATEGY];
        for i in 0..MAX_NUMBER_OF_STRATEGY {
            let arr_ref = array_ref![strategy_token_amounts, i * 8, 8];
            strategy_token_amounts_array[i] = u64::from_le_bytes(*arr_ref);
        }

        Ok(VaultStrategy {
            is_initialized: match is_initialized {
                [0] => false,
                [1] => true,
                _ => return Err(ProgramError::InvalidAccountData),
            },
            vault_account: Pubkey::new_from_array(*vault_account),
            needs_usdc_pools: needs_usdc_pools_array,
            availabilities: availabilities_array,
            strategy_token_amounts: strategy_token_amounts_array,
        })
    }
}
