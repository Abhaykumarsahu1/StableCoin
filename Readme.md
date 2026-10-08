# Stable-Coin: Remittance Stablecoin on Token-2022

An Anchor program that issues a remittance stablecoin using Solana's **Token-2022** extensions. It covers the full issuer lifecycle: fee-bearing transfers, KYC-gated accounts, on-chain metadata, mint decommissioning, and a second-generation mint that adds regulatory seizure and confidential (encrypted) transfers.

---

## Overview

The issuer needs four things from the token:

| Requirement | Mechanism |
|---|---|
| Protocol-level revenue on every transfer | `TransferFeeConfig` + `transfer_checked_with_fee` |
| New accounts unusable until KYC clears | `DefaultAccountState = Frozen` + per-account thaw |
| Wallets shouldn't trust an off-chain registry | `MetadataPointer` (self-referencing) + on-chain `TokenMetadata` |
| Ability to decommission the token | `MintCloseAuthority` |

A later regulatory scenario adds two more requirements on the same mint: **seizure from sanctioned wallets** (`PermanentDelegate`) and **hidden transfer amounts** (`ConfidentialTransfer`). Because confidential transfers cannot be added to an existing mint, v2 is a **re-issue** carrying forward the full v1 extension set.

---

## Mint Designs

### v1 mint (`create_mint`)

| Extension | Purpose |
|---|---|
| `TransferFeeConfig` | Issuer revenue on every transfer |
| `MetadataPointer` | Points at the mint itself (no external registry) |
| `DefaultAccountState` (Frozen) | New token accounts start frozen pending KYC |
| `MintCloseAuthority` | Allows closing the mint when supply is zero |

### v2 mint (`create_mint_v2`)

Everything in v1, plus:

| Extension | Purpose |
|---|---|
| `PermanentDelegate` | Seizure authority over all token accounts |
| `ConfidentialTransferMint` | Encrypted balances/transfers, `auto_approve_new_accounts = false` (manual approval policy) |
| `ConfidentialTransferFeeConfig` | Required so fees can be collected on confidential transfers |

### Initialization order

All extension-init instructions run **before** `InitializeMint`; the mint account is sized once with `ExtensionType::try_calculate_account_len`:

```
create_account
  -> TransferFeeConfig init
  -> MetadataPointer init
  -> DefaultAccountState init
  -> MintCloseAuthority init
  -> [v2] PermanentDelegate init
  -> [v2] ConfidentialTransferMint init
  -> [v2] ConfidentialTransferFeeConfig init
  -> InitializeMint2
  -> TokenMetadata initialize   (variable-length; appended after the mint exists)
```

`TokenMetadata` is variable-length, so it is not part of the pre-calculated size. Lamports for it are pre-funded at account creation and the metadata is initialized after `InitializeMint2`.

---

## Instruction Reference

| Instruction | Signer(s) | Description |
|---|---|---|
| `create_mint` | payer, mint keypair, authority | Creates the v1 mint |
| `transfer` | token owner | Fee-aware transfer via `transfer_checked_with_fee` |
| `thaw_account` | freeze authority | Thaws one token account after KYC |
| `update_default_state` | freeze authority | Mint-level change to the default state of **future** accounts |
| `close_mint` | close authority | Closes the mint (supply must be 0) |
| `create_mint_v2` | payer, mint keypair, authority | Creates the v2 mint (seizure + confidential) |
| `seize` | permanent delegate | Moves **public** balance from a source account to the treasury |
| `create_ata` | any payer | Permissionless ATA creation for any owner |
| `approve_confidential_account` | confidential-transfer authority | Manual approval of a configured account |

Confidential Configure / Deposit / ApplyPending / Transfer / Withdraw are client-side (see [below](#confidential-transfer-lifecycle)).

---

## Project Structure

```
programs/stable-coin/
├── Cargo.toml
└── src/
    ├── lib.rs                  # #[program] entrypoints
    ├── constants.rs            # metadata length limits
    ├── error.rs                # custom errors
    ├── utils.rs                # StateWithExtensions readers, fee helper
    ├── instructions.rs         # module declarations / re-exports
    └── instructions/
        ├── create_mint.rs      # v1 + v2 mint issuance (shared helper)
        ├── transfer.rs         # transfer_checked_with_fee
        ├── thaw.rs             # thaw_account, update_default_state
        ├── close_mint.rs       # MintCloseAuthority close
        ├── seize.rs            # PermanentDelegate seizure
        └── confidential.rs     # create_ata, approve_confidential_account
client/
└── confidential_flow.rs        # client-side confidential lifecycle (reference)
tests/
```
---