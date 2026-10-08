use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::token_2022::{
    spl_token_2022::{
        self,
        extension::{
            permanent_delegate::PermanentDelegate, transfer_fee::instruction::transfer_checked_with_fee,
            BaseStateWithExtensions, StateWithExtensions,
        },
        state::{AccountState, Mint},
    },
    Token2022,
};

use crate::{error::ErrorCode, utils::*};

#[derive(Accounts)]
pub struct Seize<'info> {
    /// The mint's permanent delegate
    pub delegate: Signer<'info>,
    /// CHECK: sanctioned wallet's token account
    #[account(mut)]
    pub source: UncheckedAccount<'info>,
    /// CHECK: validated through StateWithExtensions
    pub mint: UncheckedAccount<'info>,
    /// CHECK: issuer treasury token account (must itself be thawed)
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
}

/// Moves the PUBLIC (non-confidential) balance only. Confidential available/pending
/// balances are ciphertexts and are outside the delegate's reach — see README gap notes.
pub fn handle_seize(ctx: Context<Seize>, amount: u64) -> Result<()> {
    let a = &ctx.accounts;
    let mint_info = a.mint.to_account_info();
    assert_t22(&mint_info)?;
    let decimals = {
        let data = mint_info.try_borrow_data()?;
        let mint = StateWithExtensions::<Mint>::unpack(&data)?;
        let pd = mint.get_extension::<PermanentDelegate>()?;
        require!(
            Option::<Pubkey>::from(pd.delegate) == Some(a.delegate.key()),
            ErrorCode::NotPermanentDelegate
        );
        mint.base.decimals
    };

    let src = read_token_account(&a.source.to_account_info())?;
    require_keys_eq!(src.mint, a.mint.key(), ErrorCode::MintMismatch);
    // Token-2022 rejects transfers out of frozen accounts even for the permanent delegate:
    // issuer must thaw (freeze authority) first, e.g. in the same transaction.
    require!(src.state == AccountState::Initialized, ErrorCode::AccountFrozen);
    require!(src.amount >= amount, ErrorCode::InsufficientFunds);

    let fee = expected_fee(&mint_info, amount)?;
    let ix = transfer_checked_with_fee(
        &spl_token_2022::ID,
        &a.source.key(),
        &a.mint.key(),
        &a.treasury.key(),
        &a.delegate.key(), // authority = permanent delegate, not the owner
        &[],
        amount,
        decimals,
        fee,
    )?;
    invoke(
        &ix,
        &[
            a.source.to_account_info(),
            mint_info,
            a.treasury.to_account_info(),
            a.delegate.to_account_info(),
            a.token_program.to_account_info(),
        ],
    )?;
    Ok(())
}