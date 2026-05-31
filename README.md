# Solana Rental Escrow

Solana Rental Escrow is a minimal native Rust program for peer-to-peer rental deposits on Solana devnet.

Public repo: https://github.com/aisoh877/solana-rental-escrow

Program ID prepared for deployment: `Bo556kwX6HB4RKteyPsRY8F7SxQjzJanLSFNWdbiF8j3`

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

After deployment, replace `PROGRAM_ID` below.

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

Devnet links will be added after deployment:

- Program: `Bo556kwX6HB4RKteyPsRY8F7SxQjzJanLSFNWdbiF8j3`
- Create listing transaction: TBD
- Book rental transaction: TBD
- Complete rental transaction: TBD

Deployment status:

- SBF build is complete.
- Devnet deploy is pending devnet SOL. CLI airdrop was rate-limited, the Solana Foundation web faucet required Cloudflare captcha, and `devnetfaucet.org` rejected the GitHub account with `No eligible repository found in Solana ecosystem`.
