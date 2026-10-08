use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::{
    associated_token::{create_idempotent, AssociatedToken, Create},
    token_2022::{
        spl_token_2022::{self, extension::confidential_transfer::instruction::approve_account},
        Token2022,
    },
};

/// ATA creation is permissionless: ANY payer can create the account for ANY owner.
/// It only allocates the account (incl. the ConfidentialTransferAccount extension space);
/// it does NOT configure it. ConfigureAccount is owner-only and happens client-side.
#[derive(Accounts)]
pub struct CreateAta<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: wallet that will own the ATA. Does not sign.
    pub owner: UncheckedAccount<'info>,
    /// CHECK: the ATA address, derived/validated by the ATA program
    #[account(mut)]
    pub ata: UncheckedAccount<'info>,
    /// CHECK: validated by Token-2022 during the ATA CPI
    pub mint: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_create_ata(ctx: Context<CreateAta>) -> Result<()> {
    let a = &ctx.accounts;
    create_idempotent(CpiContext::new(
        a.associated_token_program.key(),
        Create {
            payer: a.payer.to_account_info(),
            associated_token: a.ata.to_account_info(),
            authority: a.owner.to_account_info(),
            mint: a.mint.to_account_info(),
            system_program: a.system_program.to_account_info(),
            token_program: a.token_program.to_account_info(),
        },
    ))?;
    Ok(())
}

/// approve_policy = manual: the issuer (confidential-transfer mint authority) must approve
/// each account AFTER the owner configured it and BEFORE confidential ops succeed.
#[derive(Accounts)]
pub struct ApproveConfidentialAccount<'info> {
    pub authority: Signer<'info>,
    /// CHECK: validated by Token-2022 during the CPI
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    /// CHECK: validated by Token-2022 during the CPI
    pub mint: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
}

pub fn handle_approve_account(ctx: Context<ApproveConfidentialAccount>) -> Result<()> {
    let a = &ctx.accounts;
    let ix = approve_account(
        &spl_token_2022::ID,
        &a.token_account.key(),
        &a.mint.key(),
        &a.authority.key(),
        &[],
    )?;
    invoke(
        &ix,
        &[
            a.token_account.to_account_info(),
            a.mint.to_account_info(),
            a.authority.to_account_info(),
            a.token_program.to_account_info(),
        ],
    )?;
    Ok(())
}