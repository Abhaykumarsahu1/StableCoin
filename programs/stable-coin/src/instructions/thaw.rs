use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::token_2022::{spl_token_2022, Token2022};
use spl_token_2022::{
    extension::default_account_state::instruction::update_default_account_state,
    instruction::thaw_account,
    state::AccountState,
};

use crate::{error::ErrorCode, utils::*};

#[derive(Accounts)]
pub struct ThawAccount<'info> {
    pub freeze_authority: Signer<'info>,
    /// CHECK: validated through StateWithExtensions
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    /// CHECK: validated through StateWithExtensions
    pub mint: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
}

/// Per-account KYC clearance. Does NOT touch the mint's DefaultAccountState.
pub fn handle_thaw_account(ctx: Context<ThawAccount>) -> Result<()> {
    let a = &ctx.accounts;
    let mint = read_mint(&a.mint.to_account_info())?;
    let acct = read_token_account(&a.token_account.to_account_info())?;

    require!(mint.freeze_authority == Some(a.freeze_authority.key()), ErrorCode::NotFreezeAuthority);
    require_keys_eq!(acct.mint, a.mint.key(), ErrorCode::MintMismatch);
    require!(acct.state == AccountState::Frozen, ErrorCode::AccountNotFrozen);

    let ix = thaw_account(
        &spl_token_2022::ID,
        &a.token_account.key(),
        &a.mint.key(),
        &a.freeze_authority.key(),
        &[],
    )?;
    invoke(
        &ix,
        &[
            a.token_account.to_account_info(),
            a.mint.to_account_info(),
            a.freeze_authority.to_account_info(),
            a.token_program.to_account_info(),
        ],
    )?;
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateDefaultState<'info> {
    pub freeze_authority: Signer<'info>,
    /// CHECK: validated by Token-2022 during the CPI
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
}

/// Mint-level policy change. Only affects accounts created AFTER this call.
pub fn handle_update_default_state(ctx: Context<UpdateDefaultState>, frozen: bool) -> Result<()> {
    let a = &ctx.accounts;
    let state = if frozen { AccountState::Frozen } else { AccountState::Initialized };
    let ix = update_default_account_state(
        &spl_token_2022::ID,
        &a.mint.key(),
        &a.freeze_authority.key(),
        &[],
        &state,
    )?;
    invoke(
        &ix,
        &[
            a.mint.to_account_info(),
            a.freeze_authority.to_account_info(),
            a.token_program.to_account_info(),
        ],
    )?;
    Ok(())
}