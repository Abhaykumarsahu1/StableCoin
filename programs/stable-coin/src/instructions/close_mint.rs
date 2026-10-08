use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::token_2022::{
    spl_token_2022::{
        self,
        extension::{mint_close_authority::MintCloseAuthority, BaseStateWithExtensions, StateWithExtensions},
        instruction::close_account,
        state::Mint,
    },
    Token2022,
};

use crate::{error::ErrorCode, utils::*};

#[derive(Accounts)]
pub struct CloseMint<'info> {
    pub close_authority: Signer<'info>,
    /// CHECK: validated through StateWithExtensions
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
    /// CHECK: receives the reclaimed rent
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
}

pub fn handle_close_mint(ctx: Context<CloseMint>) -> Result<()> {
    let a = &ctx.accounts;
    let mint_info = a.mint.to_account_info();
    assert_t22(&mint_info)?;
    {
        let data = mint_info.try_borrow_data()?;
        let mint = StateWithExtensions::<Mint>::unpack(&data)?;
        require!(mint.base.supply == 0, ErrorCode::SupplyNotZero);
        let ext = mint.get_extension::<MintCloseAuthority>()?;
        require!(
            Option::<Pubkey>::from(ext.close_authority) == Some(a.close_authority.key()),
            ErrorCode::NotCloseAuthority
        );
    }
    let ix = close_account(
        &spl_token_2022::ID,
        &a.mint.key(),
        &a.destination.key(),
        &a.close_authority.key(),
        &[],
    )?;
    invoke(
        &ix,
        &[
            mint_info,
            a.destination.to_account_info(),
            a.close_authority.to_account_info(),
            a.token_program.to_account_info(),
        ],
    )?;
    Ok(())
}