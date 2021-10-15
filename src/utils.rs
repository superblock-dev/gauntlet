use solana_program::{
    account_info::AccountInfo,
    bpf_loader_upgradeable::{UpgradeableLoaderState},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_option::COption,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
};

use crate::error::GauntletError;
use spl_token::instruction::AuthorityType::AccountOwner;
use std::result::Result;
use std::str::FromStr;

// token account의 owner를 변경하는 instruction을 생성 및 invoke
pub fn change_token_account_owner<'a>(
    token_account: &AccountInfo<'a>,
    current_owner: &AccountInfo<'a>,
    new_owner: &Pubkey,
) -> ProgramResult {
    let new_authority: COption<Pubkey> = Some(new_owner).cloned().into();
    let data = spl_token::instruction::TokenInstruction::SetAuthority {
        authority_type: AccountOwner,
        new_authority,
    }
    .pack();

    let accounts = vec![
        AccountMeta::new(*token_account.key, false),
        AccountMeta::new_readonly(*current_owner.key, true),
    ];

    let ix = &Instruction {
        program_id: spl_token::id(),
        accounts,
        data,
    };

    invoke(ix, &[token_account.clone(), current_owner.clone()])?;
    Ok(())
}

pub fn transfer_token<'a>(
    token_program_id: &Pubkey,
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
    amount: u64,
) -> ProgramResult {
    let data = spl_token::instruction::TokenInstruction::Transfer { amount }.pack();

    let accounts = vec![
        AccountMeta::new(*from.key, false),
        AccountMeta::new(*to.key, false),
        AccountMeta::new_readonly(*owner.key, true),
    ];

    let ix = &Instruction {
        program_id: *token_program_id,
        accounts,
        data,
    };

    invoke(ix, &[from.clone(), to.clone(), owner.clone()])?;
    Ok(())
}

pub fn transfer_token_signed<'a>(
    token_program_id: &Pubkey,
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
    amount: u64,
) -> ProgramResult {
    let data = spl_token::instruction::TokenInstruction::Transfer { amount }.pack();

    let accounts = vec![
        AccountMeta::new(*from.key, false),
        AccountMeta::new(*to.key, false),
        AccountMeta::new_readonly(*owner.key, true),
    ];

    let ix = &Instruction {
        program_id: *token_program_id,
        accounts,
        data,
    };

    invoke_signed(
        ix,
        &[from.clone(), to.clone(), owner.clone()],
        &[&[&b"glt"[..], &[255]]],
    )?;
    Ok(())
}

pub fn create_pda_account<'a>(
    payer: &AccountInfo<'a>,
    space: usize,
    owner: &Pubkey,
    system_program: &AccountInfo<'a>,
    new_pda_account: &AccountInfo<'a>,
    new_pda_signer_seeds: &[&[u8]],
) -> ProgramResult {
    let rent = Rent::default();
    if new_pda_account.lamports() > 0 {
        let required_lamports = rent
            .minimum_balance(space)
            .max(1)
            .saturating_sub(new_pda_account.lamports());

        if required_lamports > 0 {
            invoke(
                &system_instruction::transfer(payer.key, new_pda_account.key, required_lamports),
                &[
                    payer.clone(),
                    new_pda_account.clone(),
                    system_program.clone(),
                ],
            )?;
        }

        invoke_signed(
            &system_instruction::allocate(new_pda_account.key, space as u64),
            &[new_pda_account.clone(), system_program.clone()],
            &[new_pda_signer_seeds],
        )?;

        invoke_signed(
            &system_instruction::assign(new_pda_account.key, owner),
            &[new_pda_account.clone(), system_program.clone()],
            &[new_pda_signer_seeds],
        )
    } else {
        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                new_pda_account.key,
                rent.minimum_balance(space).max(1),
                space as u64,
                owner,
            ),
            &[
                payer.clone(),
                new_pda_account.clone(),
                system_program.clone(),
            ],
            &[new_pda_signer_seeds],
        )
    }
}

pub fn get_program_upgrade_authority(
    upgradable_loader_state: &UpgradeableLoaderState,
) -> Result<Option<Pubkey>, ProgramError> {
    let upgrade_authority = match upgradable_loader_state {
        UpgradeableLoaderState::ProgramData {
            slot: _,
            upgrade_authority_address,
        } => *upgrade_authority_address,
        _ => return Err(ProgramError::InvalidAccountData),
    };

    Ok(upgrade_authority)
}

pub const STAKING_PROGRAM_ID: [&str; 3] = [
    "EhhTKczWMGQt46ynNeRX1WfeagwwJd7ufHvCDjRxjo5Q",
    "CBuCnLe26faBpcBP2fktp4rp8abpcAnTWft6ZrP5Q4T",
    "9KEPoZmtHUrBbhWN1v1KWLMkkvwY6WLtAVUCPRtRjP4z",
];
pub fn check_staking_program_id(program_id: &AccountInfo) -> ProgramResult {
    for i in 0..3 {
        if Pubkey::from_str(STAKING_PROGRAM_ID[i]).unwrap() == *program_id.key {
            return Ok(());
        }
    }
    Err(GauntletError::InvalidProgramId.into())
}
pub const POOL_PROGRAM_ID: [&str; 3] = [
    "RVKd61ztZW9GUwhRbbLoYVRE5Xf1B2tVscKqwZqXgEr",
    "27haf8L6oxUeXrHrgEgsexjSY5hbVUWEmvv9Nyxg8vQv",
    "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
];
pub fn check_pool_program_id(program_id: &AccountInfo) -> ProgramResult {
    for i in 0..3 {
        if Pubkey::from_str(POOL_PROGRAM_ID[i]).unwrap() == *program_id.key {
            return Ok(());
        }
    }
    Err(GauntletError::InvalidProgramId.into())
}
