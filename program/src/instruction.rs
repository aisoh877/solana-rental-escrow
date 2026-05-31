use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::state::{ESCROW_SEED, LISTING_SEED};

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub enum RentalInstruction {
    CreateListing {
        listing_id: u64,
        deposit_lamports: u64,
        rental_fee_lamports: u64,
        max_duration_slots: u64,
    },
    BookRental {
        start_slot: u64,
        end_slot: u64,
    },
    CompleteRental,
    CancelListing,
    FlagDispute,
}

pub fn listing_pda(program_id: &Pubkey, owner: &Pubkey, listing_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[LISTING_SEED, owner.as_ref(), &listing_id.to_le_bytes()],
        program_id,
    )
}

pub fn escrow_pda(program_id: &Pubkey, listing: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[ESCROW_SEED, listing.as_ref()], program_id)
}

pub fn create_listing(
    program_id: Pubkey,
    owner: Pubkey,
    listing_id: u64,
    deposit_lamports: u64,
    rental_fee_lamports: u64,
    max_duration_slots: u64,
) -> Instruction {
    let (listing, _) = listing_pda(&program_id, &owner, listing_id);
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(owner, true),
            AccountMeta::new(listing, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: encode(&RentalInstruction::CreateListing {
            listing_id,
            deposit_lamports,
            rental_fee_lamports,
            max_duration_slots,
        }),
    }
}

pub fn book_rental(
    program_id: Pubkey,
    owner: Pubkey,
    renter: Pubkey,
    listing_id: u64,
    start_slot: u64,
    end_slot: u64,
) -> Instruction {
    let (listing, _) = listing_pda(&program_id, &owner, listing_id);
    let (escrow, _) = escrow_pda(&program_id, &listing);
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(renter, true),
            AccountMeta::new(owner, false),
            AccountMeta::new(listing, false),
            AccountMeta::new(escrow, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: encode(&RentalInstruction::BookRental {
            start_slot,
            end_slot,
        }),
    }
}

pub fn complete_rental(
    program_id: Pubkey,
    owner: Pubkey,
    renter: Pubkey,
    listing_id: u64,
) -> Instruction {
    let (listing, _) = listing_pda(&program_id, &owner, listing_id);
    let (escrow, _) = escrow_pda(&program_id, &listing);
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(owner, true),
            AccountMeta::new(renter, false),
            AccountMeta::new(listing, false),
            AccountMeta::new(escrow, false),
        ],
        data: encode(&RentalInstruction::CompleteRental),
    }
}

pub fn cancel_listing(program_id: Pubkey, owner: Pubkey, listing_id: u64) -> Instruction {
    let (listing, _) = listing_pda(&program_id, &owner, listing_id);
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(owner, true),
            AccountMeta::new(listing, false),
        ],
        data: encode(&RentalInstruction::CancelListing),
    }
}

pub fn flag_dispute(
    program_id: Pubkey,
    signer: Pubkey,
    owner: Pubkey,
    listing_id: u64,
) -> Instruction {
    let (listing, _) = listing_pda(&program_id, &owner, listing_id);
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(listing, false),
        ],
        data: encode(&RentalInstruction::FlagDispute),
    }
}

fn encode(instruction: &RentalInstruction) -> Vec<u8> {
    to_vec(instruction).expect("serialize rental instruction")
}
