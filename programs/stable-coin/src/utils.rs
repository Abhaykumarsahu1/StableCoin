use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022::{
    self,
    extension::{
        transfer_fee::TransferFeeConfig, BaseStateWithExtensions, StateWithExtensions,
    },
    state::{Account as TokenAccount, AccountState, Mint},
};

use crate::error::ErrorCode;

pub fn assert_t22(info: &AccountInfo) -> Result<()> {
    require_keys_eq!(*info.owner, spl_token_2022::ID, ErrorCode::NotToken2022);
    Ok(())
}

pub struct TokenAccountView {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub state: AccountState,
}

pub struct MintView {
    pub decimals: u8,
    pub supply: u64,
    pub freeze_authority: Option<Pubkey>,
}

/// Never `Account::unpack` — extensions make the account longer than 165 bytes.
pub fn read_token_account(info: &AccountInfo) -> Result<TokenAccountView> {
    assert_t22(info)?;
    let data = info.try_borrow_data()?;
    let acct = StateWithExtensions::<TokenAccount>::unpack(&data)?;
    Ok(TokenAccountView {
        mint: acct.base.mint,
        owner: acct.base.owner,
        amount: acct.base.amount,
        state: acct.base.state,
    })
}

pub fn read_mint(info: &AccountInfo) -> Result<MintView> {
    assert_t22(info)?;
    let data = info.try_borrow_data()?;
    let mint = StateWithExtensions::<Mint>::unpack(&data)?;
    Ok(MintView {
        decimals: mint.base.decimals,
        supply: mint.base.supply,
        freeze_authority: Option::<Pubkey>::from(mint.base.freeze_authority),
    })
}

/// Task 2: fee is derived from the epoch-aware config (older/newer fee schedule),
/// never from a cached bps/max value.
pub fn expected_fee(mint_info: &AccountInfo, amount: u64) -> Result<u64> {
    assert_t22(mint_info)?;
    let data = mint_info.try_borrow_data()?;
    let mint = StateWithExtensions::<Mint>::unpack(&data)?;
    let cfg = mint.get_extension::<TransferFeeConfig>()?;
    let epoch = Clock::get()?.epoch;
    cfg.calculate_epoch_fee(epoch, amount)
        .ok_or_else(|| error!(ErrorCode::FeeOverflow))
}