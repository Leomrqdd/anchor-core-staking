# anchor-core-staking

Anchor program on Solana for staking **MPL Core** NFTs and earning SPL token rewards.

Program ID: `FdQ9QdcVkYVqqGwCX8ghKiZAiDzYdbuC9c63rhuqVCmf`

## Overview

Each staked asset is frozen via MPL Core's `FreezeDelegate` plugin and tagged with `staked`/`staked_at` attributes. Rewards accrue per day staked and are minted as an SPL token when the user calls `claim_rewards`. Claiming resets the timer, so rewards are non-cumulative across claims.

## Instructions

| Instruction | Description |
|---|---|
| `initialize` | Creates the `Config` PDA and the rewards mint for a given collection. Sets `rewards_bps` and `freeze_period`. |
| `create_collection` | Creates an MPL Core collection with the program's PDA as update authority. |
| `mint_asset` | Mints an MPL Core asset into the collection. |
| `stake` | Freezes the asset (FreezeDelegate) and writes `staked = true` + `staked_at = <unix_ts>` attributes. Increments `staked_count` on the collection. |
| `unstake` | Unfreezes the asset after the freeze period has elapsed. Decrements `staked_count`. |
| `claim_rewards` | Mints `days_staked × rewards_bps / 10000` tokens (6 decimals) to the owner's ATA, then resets `staked_at` to now. |

## Reward formula

```
amount = floor(days_staked × rewards_bps × 10^6 / 10_000)
```

Example: 8 days staked, `rewards_bps = 500` → `8 × 500 × 1_000_000 / 10_000 = 400_000` base units.

## Accounts

**Config** (PDA: `["config", collection]`)

| Field | Type | Description |
|---|---|---|
| `rewards_bps` | `u16` | Daily reward rate in basis points |
| `freeze_period` | `u16` | Minimum days before unstake is allowed |
| `reward_bump` | `u8` | Bump for the rewards mint PDA |
| `bump` | `u8` | Bump for the config PDA |

**Rewards mint** (PDA: `["reward_mint", config]`) — SPL token with 6 decimals, minted by the config PDA.

**Update authority** (PDA: `["update_authority", collection]`) — signer-only PDA used for MPL Core CPI calls.

## Prerequisites

- Rust + Anchor `0.31.1`
- Solana CLI
- Yarn

## Build & test

```bash
anchor build
cargo test
```

Tests run against [LiteSVM](https://github.com/LiteSVM/litesvm) — no local validator needed. The `mpl_core.so` fixture is bundled under `tests/fixtures/`.
