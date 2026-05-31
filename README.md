# Solana Rental Escrow

Solana Rental Escrow is a minimal native Rust program for peer-to-peer rental deposits on Solana devnet.

Public repo: https://github.com/aisoh877/solana-rental-escrow

Devnet program ID: `Bo556kwX6HB4RKteyPsRY8F7SxQjzJanLSFNWdbiF8j3`

Everyday scenario: a neighbor rents out a camera, bike, tool, or other daily item. In the traditional flow, the owner asks for a cash deposit, the renter has little visibility into refund rules, and a dispute turns into a private chat. This program moves the core agreement on-chain: the owner publishes rental terms, the renter locks a deposit into a PDA escrow, and the owner can return the deposit when the item comes back.

## Why This Belongs On-Chain

Traditional friction:

- Deposits are held by one party, so refund trust is asymmetric.
- Rental terms are often informal and hard to verify later.
- Local item rental markets need simple, low-cost guarantees before strangers can transact.

Solana model:

- A listing PDA stores the owner, renter, fee, deposit, allowed duration, and state.
- An escrow PDA holds the renter deposit under program control.
- The rental fee transfers directly to the owner when booking.
- Completion returns the escrowed deposit to the renter.
- Either party can flag a dispute, freezing the deposit state for off-chain mediation instead of silently releasing funds.

Tradeoffs:

- This MVP does not judge physical-world damage on-chain.
- Dispute resolution is intentionally conservative: it freezes funds instead of pretending the program can verify item condition.
- It uses native SOL on devnet to stay minimal; production versions could add SPL token support and a DAO/local mediator path.

## Program Design

Accounts:

- `listing`: PDA derived from `["listing", owner, listing_id]`; stores the rental terms and state.
- `escrow`: PDA derived from `["escrow", listing]`; holds the renter deposit while booked.
- `owner`: creates the listing, receives the rental fee, and completes/cancels the rental.
- `renter`: books the listing, funds the deposit, and receives the returned deposit.

Instructions:

- `CreateListing`: owner creates terms with deposit, fee, and max duration.
- `BookRental`: renter pays the fee and locks the deposit in escrow.
- `CompleteRental`: owner returns the escrowed deposit to the renter.
- `CancelListing`: owner cancels an unbooked listing.
- `FlagDispute`: owner or renter freezes a booked rental as disputed.

## Build And Test

```bash
source ~/.cargo/env
cargo test
```

Verified locally on 2026-05-31:

- `cargo test` passed.
- `cargo build-sbf --manifest-path program/Cargo.toml` passed.

Build SBF:

```bash
source ~/.cargo/env
cargo build-sbf --manifest-path program/Cargo.toml
```

## Devnet Deployment

```bash
solana config set --url devnet
solana-keygen new -o ~/.config/solana/id.json
solana airdrop 2
source ~/.cargo/env
cargo build-sbf --manifest-path program/Cargo.toml
solana program deploy target/deploy/rental_escrow_program.so
```

Deployed program:

- Program ID: `Bo556kwX6HB4RKteyPsRY8F7SxQjzJanLSFNWdbiF8j3`
- Explorer: https://explorer.solana.com/address/Bo556kwX6HB4RKteyPsRY8F7SxQjzJanLSFNWdbiF8j3?cluster=devnet

Use the deployed program ID in the commands below.

## CLI Usage

Derive addresses:

```bash
cargo run -p rental-escrow-cli -- \
  --program-id PROGRAM_ID \
  pda --owner OWNER_PUBKEY --listing-id 1
```

Create a listing:

```bash
cargo run -p rental-escrow-cli -- \
  --program-id PROGRAM_ID \
  create --listing-id 1 --deposit-sol 0.1 --fee-sol 0.01 --max-duration-slots 50000
```

Book a rental:

```bash
cargo run -p rental-escrow-cli -- \
  --program-id PROGRAM_ID \
  book --owner OWNER_PUBKEY --listing-id 1 --start-slot START --end-slot END
```

Complete and return deposit:

```bash
cargo run -p rental-escrow-cli -- \
  --program-id PROGRAM_ID \
  complete --owner OWNER_PUBKEY --renter RENTER_PUBKEY --listing-id 1
```

Show listing state:

```bash
cargo run -p rental-escrow-cli -- \
  --program-id PROGRAM_ID \
  show --owner OWNER_PUBKEY --listing-id 1
```

## Submission Evidence

- Program: https://explorer.solana.com/address/Bo556kwX6HB4RKteyPsRY8F7SxQjzJanLSFNWdbiF8j3?cluster=devnet
- Listing PDA: `Ff2a85aT7AAPMZSESi719impek84iGjLjwg1tUx2AUG9`
- Escrow PDA: `6BxX2878KihoToDeaNajfzcdm31wgwPFwPVCCiGkBFsV`
- Create listing transaction: https://explorer.solana.com/tx/9dyeTofHTouELJM9NyzRSrMiY8PF1CCGXsPnrBrTA33s4LEqruE4hBGMRqWnpLuArjbbGpEtZQGYv4wCgeTJDCz?cluster=devnet
- Book rental transaction: https://explorer.solana.com/tx/26MzKpv2MMUmHWYxhkC6e9jNoH8RF9m8p89MM2PyGHrAukgTPNV3FpL5XdV1A7bcwQvL1BZWYAqGwAcoSykSrTkD?cluster=devnet
- Complete rental transaction: https://explorer.solana.com/tx/3Q8LzJTbL7Z3TeTjEFupPC1Dn5HuUkFMuw4eavQi3oM2WNyMjFFesMSRMH1zCax1PKF9ExSJc8ctagbRha1dzd81?cluster=devnet

Final listing state after the demo:

```text
owner: Dcf6a6ag8MrrQd5Ty8ukqoRmumLxk49MPAEfSeCVNozN
renter: C2ownhAaUvhGGK8psR6FDjH8xyz1tmsq8cdiJd7zk5Mn
listing_id: 1
deposit_lamports: 50000000
rental_fee_lamports: 10000000
max_duration_slots: 1000
status: completed
```
