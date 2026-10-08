use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::token_2022::{spl_token_2022, Token2022};
use spl_token_2022::{extension::transfer_fee::instruction::transfer_checked_with_fee, state::AccountState};

use crate::{error::ErrorCode, utils::*};

#[derive(Accounts)]
pub struct TransferWithFee<'info> {
    /// CHECK: validated through StateWithExtensions in the handler
    #[account(mut)]
    pub source: UncheckedAccount<'info>,
    /// CHECK: validated through StateWithExtensions in the handler
    pub mint: UncheckedAccount<'info>,
    /// CHECK: destination token account; Token-2022 validates it during the CPI
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token2022>,
}

pub fn handle_transfer(ctx: Context<TransferWithFee>, amount: u64) -> Result<()> {
    let a = &ctx.accounts;
    let mint_info = a.mint.to_account_info();
    let src_info = a.source.to_account_info();

    let mint = read_mint(&mint_info)?;
    let src = read_token_account(&src_info)?;
    require_keys_eq!(src.mint, a.mint.key(), ErrorCode::MintMismatch);
    require_keys_eq!(src.owner, a.owner.key(), ErrorCode::OwnerMismatch);
    require!(src.state == AccountState::Initialized, ErrorCode::AccountFrozen);
    require!(src.amount >= amount, ErrorCode::InsufficientFunds);

    // fee comes from calculate_epoch_fee(current_epoch, amount), not a cached rate
    let fee = expected_fee(&mint_info, amount)?;

    let ix = transfer_checked_with_fee(
        &spl_token_2022::ID,
        &a.source.key(),
        &a.mint.key(),
        &a.destination.key(),
        &a.owner.key(),
        &[],
        amount,
        mint.decimals,
        fee,
    )?;
    invoke(
        &ix,
        &[
            src_info,
            mint_info,
            a.destination.to_account_info(),
            a.owner.to_account_info(),
            a.token_program.to_account_info(),
        ],
    )?;
    Ok(())
}