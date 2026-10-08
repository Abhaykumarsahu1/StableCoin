use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::system_program::{create_account, CreateAccount};
use anchor_spl::token_2022::{spl_token_2022, Token2022};
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token_2022::{
    extension::{
        confidential_transfer, confidential_transfer_fee, default_account_state,
        metadata_pointer, transfer_fee, ExtensionType,
    },
    instruction as t22,
    state::{AccountState, Mint},
};
// NOTE: path differs by spl-token-2022 version (solana_zk_sdk vs solana_zk_token_sdk)
use spl_token_2022::solana_zk_sdk::encryption::pod::elgamal::PodElGamalPubkey;
use spl_token_metadata_interface::{instruction as md_ix, state::TokenMetadata};

use crate::{constants::*, error::ErrorCode};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MintParams {
    pub decimals: u8,
    pub fee_basis_points: u16,
    pub max_fee: u64,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct V2Params {
    /// ElGamal pubkey that can decrypt every confidential transfer amount (read-only visibility)
    pub auditor_elgamal_pubkey: Option<[u8; 32]>,
    /// ElGamal pubkey that receives/decrypts withheld *confidential* fees
    pub withdraw_withheld_elgamal_pubkey: [u8; 32],
}

#[derive(Accounts)]
pub struct CreateMint<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// New mint keypair; signs its own creation.
    #[account(mut)]
    pub mint: Signer<'info>,
    /// Issuer. Used for every authority on the mint (kept as one key for simplicity).
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handle_create_mint(ctx: Context<CreateMint>, p: MintParams) -> Result<()> {
    issue_mint(&ctx, &p, None)
}

/// Task 5: same base extension set as v1, plus PermanentDelegate, ConfidentialTransferMint
/// (manual approval) and ConfidentialTransferFee (required because the mint carries a transfer fee).
pub fn handle_create_mint_v2(ctx: Context<CreateMint>, p: MintParams, c: V2Params) -> Result<()> {
    issue_mint(&ctx, &p, Some(&c))
}

fn issue_mint(ctx: &Context<CreateMint>, p: &MintParams, v2: Option<&V2Params>) -> Result<()> {
    require!(
        p.name.len() <= MAX_NAME_LEN && p.symbol.len() <= MAX_SYMBOL_LEN && p.uri.len() <= MAX_URI_LEN,
        ErrorCode::MetadataTooLong
    );

    let payer = ctx.accounts.payer.to_account_info();
    let mint = ctx.accounts.mint.to_account_info();
    let authority = ctx.accounts.authority.to_account_info();
    let token_program = ctx.accounts.token_program.to_account_info();
    let system_program = ctx.accounts.system_program.to_account_info();

    let tp = &spl_token_2022::ID;
    let m = mint.key;
    let a = authority.key;
    let infos = [mint.clone(), token_program.clone()];

    // ---- sizing: fixed-size extensions only; TokenMetadata is variable-length and is
    // appended after InitializeMint (pre-fund lamports for it instead of pre-allocating).
    let mut exts = vec![
        ExtensionType::TransferFeeConfig,
        ExtensionType::MetadataPointer,
        ExtensionType::DefaultAccountState,
        ExtensionType::MintCloseAuthority,
    ];
    if v2.is_some() {
        exts.extend([
            ExtensionType::PermanentDelegate,
            ExtensionType::ConfidentialTransferMint,
            ExtensionType::ConfidentialTransferFeeConfig,
        ]);
    }
    let space = ExtensionType::try_calculate_account_len::<Mint>(&exts)?;

    let metadata = TokenMetadata {
        update_authority: OptionalNonZeroPubkey::try_from(Some(*a))?,
        mint: *m,
        name: p.name.clone(),
        symbol: p.symbol.clone(),
        uri: p.uri.clone(),
        additional_metadata: vec![],
    };
    let metadata_len = metadata.tlv_size_of()? + 64; // small safety margin; overfunding is harmless
    let lamports = Rent::get()?.minimum_balance(space + metadata_len);

    create_account(
        CpiContext::new(
            system_program.key(),
            CreateAccount { from: payer, to: mint.clone() },
        ),
        lamports,
        space as u64,
        tp,
    )?;

    // ---- every extension init BEFORE InitializeMint, in this order ----
    invoke(
        &transfer_fee::instruction::initialize_transfer_fee_config(
            tp, m, Some(a), Some(a), p.fee_basis_points, p.max_fee,
        )?,
        &infos,
    )?;
    invoke(
        &metadata_pointer::instruction::initialize(tp, m, Some(*a), Some(*m))?, // points at itself
        &infos,
    )?;
    invoke(
        &default_account_state::instruction::initialize_default_account_state(
            tp, m, &AccountState::Frozen,
        )?,
        &infos,
    )?;
    invoke(&t22::initialize_mint_close_authority(tp, m, Some(a))?, &infos)?;

    if let Some(c) = v2 {
        invoke(&t22::initialize_permanent_delegate(tp, m, a)?, &infos)?;

        let auditor = c.auditor_elgamal_pubkey.map(bytemuck::cast::<[u8; 32], PodElGamalPubkey>);
        invoke(
            &confidential_transfer::instruction::initialize_mint(
                tp, m, Some(*a),
                false, // auto_approve_new_accounts = false  => approve_policy = manual
                auditor,
            )?,
            &infos,
        )?;

        let fee_key: PodElGamalPubkey = bytemuck::cast(c.withdraw_withheld_elgamal_pubkey);
invoke(
    &confidential_transfer_fee::instruction::initialize_confidential_transfer_fee_config(
        tp, m, Some(*a), &fee_key,
    )?,
    &infos,
)?;
    }

    // freeze authority is mandatory for DefaultAccountState::Frozen to be usable
    invoke(&t22::initialize_mint2(tp, m, a, Some(a), p.decimals)?, &infos)?;

    // on-chain metadata (mint is its own metadata account); mint authority signs
    invoke(
        &md_ix::initialize(tp, m, a, m, a, p.name.clone(), p.symbol.clone(), p.uri.clone()),
        &[mint.clone(), authority.clone(), token_program.clone()],
    )?;
    Ok(())
}