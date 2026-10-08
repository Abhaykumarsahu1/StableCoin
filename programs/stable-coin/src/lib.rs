use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod utils;

use instructions::*;

declare_id!("EoxhrP86carJDUew52FxkKBxLJw5QQDGRYdkuougTPcR");

#[program]
pub mod stable_coin {
    use super::*;

    // Task 1
    pub fn create_mint(ctx: Context<CreateMint>, p: MintParams) -> Result<()> {
        instructions::create_mint::handle_create_mint(ctx, p)
    }
    // Task 2
    pub fn transfer(ctx: Context<TransferWithFee>, amount: u64) -> Result<()> {
        instructions::transfer::handle_transfer(ctx, amount)
    }
    // Task 4
    pub fn thaw_account(ctx: Context<ThawAccount>) -> Result<()> {
        instructions::thaw::handle_thaw_account(ctx)
    }
    pub fn update_default_state(ctx: Context<UpdateDefaultState>, frozen: bool) -> Result<()> {
        instructions::thaw::handle_update_default_state(ctx, frozen)
    }
    pub fn close_mint(ctx: Context<CloseMint>) -> Result<()> {
        instructions::close_mint::handle_close_mint(ctx)
    }
    // Task 5
    pub fn create_mint_v2(ctx: Context<CreateMint>, p: MintParams, c: V2Params) -> Result<()> {
        instructions::create_mint::handle_create_mint_v2(ctx, p, c)
    }
    pub fn seize(ctx: Context<Seize>, amount: u64) -> Result<()> {
        instructions::seize::handle_seize(ctx, amount)
    }
    // Task 6 (on-chain parts only; see client/confidential_flow.rs)
    pub fn create_ata(ctx: Context<CreateAta>) -> Result<()> {
        instructions::confidential::handle_create_ata(ctx)
    }
    pub fn approve_confidential_account(ctx: Context<ApproveConfidentialAccount>) -> Result<()> {
        instructions::confidential::handle_approve_account(ctx)
    }
}
