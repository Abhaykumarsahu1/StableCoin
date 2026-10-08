use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Account is not owned by the Token-2022 program")]
    NotToken2022,
    #[msg("Token account mint does not match the supplied mint")]
    MintMismatch,
    #[msg("Signer is not the token account owner")]
    OwnerMismatch,
    #[msg("Token account is frozen (KYC pending, or thaw it first)")]
    AccountFrozen,
    #[msg("Token account is not frozen")]
    AccountNotFrozen,
    #[msg("Insufficient token balance")]
    InsufficientFunds,
    #[msg("Fee calculation overflowed")]
    FeeOverflow,
    #[msg("Signer is not the mint freeze authority")]
    NotFreezeAuthority,
    #[msg("Signer is not the mint close authority")]
    NotCloseAuthority,
    #[msg("Signer is not the permanent delegate")]
    NotPermanentDelegate,
    #[msg("Mint supply must be zero before it can be closed")]
    SupplyNotZero,
    #[msg("Metadata field too long")]
    MetadataTooLong,
}