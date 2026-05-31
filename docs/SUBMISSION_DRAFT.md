# Superteam Submission Draft

Listing: https://superteam.fun/earn/listing/build-everyday-real-world-systems-as-on-chain-rust-programs/

Repository: https://github.com/aisoh877/solana-rental-escrow

Prepared program ID: `Bo556kwX6HB4RKteyPsRY8F7SxQjzJanLSFNWdbiF8j3`

## Project Summary

Solana Rental Escrow is a native Rust Solana program for peer-to-peer rentals of everyday items such as cameras, bikes, tools, or local equipment. It turns an informal deposit agreement into a transparent on-chain flow:

- The owner creates a listing PDA with a deposit, fee, and max rental duration.
- The renter books the item, pays the rental fee, and locks the deposit in a program-owned escrow PDA.
- The owner completes the rental to return the deposit.
- Either party can flag a dispute to freeze the rental state for off-chain mediation.

## Why This Solves Daily Friction

Traditional item rental deposits are asymmetric: the renter sends funds to the owner and must trust that the owner will return them. Terms are usually buried in chat messages, and disputes are private.

This program moves the minimal enforceable core on-chain. Deposit custody is program-controlled, terms are stored in a public account, and state transitions are signed by the appropriate party.

## Architecture

- `listing` PDA: `["listing", owner, listing_id]`
- `escrow` PDA: `["escrow", listing]`
- `CreateListing`: owner sets deposit, fee, and max duration.
- `BookRental`: renter pays fee and locks deposit.
- `CompleteRental`: owner returns deposit to renter.
- `CancelListing`: owner cancels an unbooked listing.
- `FlagDispute`: owner or renter freezes a booked rental.

## Verification

Local verification completed on 2026-05-31:

```bash
cargo test
cargo build-sbf --manifest-path program/Cargo.toml
```

Both commands passed.

GitHub Actions CI also passed:

- https://github.com/aisoh877/solana-rental-escrow/actions/runs/26701593048

## Devnet Links

Pending devnet SOL funding for deployment wallet:

- Deploy wallet: `Dcf6a6ag8MrrQd5Ty8ukqoRmumLxk49MPAEfSeCVNozN`
- Program: `Bo556kwX6HB4RKteyPsRY8F7SxQjzJanLSFNWdbiF8j3`
- Program explorer link: TBD
- Create listing transaction: TBD
- Book rental transaction: TBD
- Complete rental transaction: TBD

## Current Deployment Blocker

The code is ready for deployment, but the deploy wallet currently has `0 SOL` on devnet. Attempts made:

- `solana airdrop`: rate limited.
- Solana Foundation faucet: GitHub auth succeeded, but Cloudflare captcha requires manual browser completion.
- `devnetfaucet.org`: GitHub auth succeeded, but the account was rejected with `No eligible repository found in Solana ecosystem`.
- `devnetfaucet.org` vouch request flow: request accepted, but airdrop still failed and requires a public `Tweet for a Vouch`.
- `solfaucet.com`: returned RPC internal error and no transaction ID.
- QuickNode faucet: rejected the deploy wallet because it has insufficient mainnet SOL balance.
