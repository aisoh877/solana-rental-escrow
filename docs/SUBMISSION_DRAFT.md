# Superteam Submission Draft

Listing: https://superteam.fun/earn/listing/build-everyday-real-world-systems-as-on-chain-rust-programs/

Repository: https://github.com/aisoh877/solana-rental-escrow

Devnet program ID: `Bo556kwX6HB4RKteyPsRY8F7SxQjzJanLSFNWdbiF8j3`

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
- https://github.com/aisoh877/solana-rental-escrow/actions/runs/26701686727

## Devnet Links

- Deploy wallet: `Dcf6a6ag8MrrQd5Ty8ukqoRmumLxk49MPAEfSeCVNozN`
- Program: https://explorer.solana.com/address/Bo556kwX6HB4RKteyPsRY8F7SxQjzJanLSFNWdbiF8j3?cluster=devnet
- Listing PDA: `Ff2a85aT7AAPMZSESi719impek84iGjLjwg1tUx2AUG9`
- Escrow PDA: `6BxX2878KihoToDeaNajfzcdm31wgwPFwPVCCiGkBFsV`
- Create listing transaction: https://explorer.solana.com/tx/9dyeTofHTouELJM9NyzRSrMiY8PF1CCGXsPnrBrTA33s4LEqruE4hBGMRqWnpLuArjbbGpEtZQGYv4wCgeTJDCz?cluster=devnet
- Book rental transaction: https://explorer.solana.com/tx/26MzKpv2MMUmHWYxhkC6e9jNoH8RF9m8p89MM2PyGHrAukgTPNV3FpL5XdV1A7bcwQvL1BZWYAqGwAcoSykSrTkD?cluster=devnet
- Complete rental transaction: https://explorer.solana.com/tx/3Q8LzJTbL7Z3TeTjEFupPC1Dn5HuUkFMuw4eavQi3oM2WNyMjFFesMSRMH1zCax1PKF9ExSJc8ctagbRha1dzd81?cluster=devnet

## Demo State

Final listing state after the devnet demo:

```text
owner: Dcf6a6ag8MrrQd5Ty8ukqoRmumLxk49MPAEfSeCVNozN
renter: C2ownhAaUvhGGK8psR6FDjH8xyz1tmsq8cdiJd7zk5Mn
listing_id: 1
deposit_lamports: 50000000
rental_fee_lamports: 10000000
max_duration_slots: 1000
start_slot: 466095062
end_slot: 466095562
status: completed
```
