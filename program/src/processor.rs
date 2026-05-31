use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::{
    instruction::{escrow_pda, listing_pda, RentalInstruction},
    state::{Listing, RentalStatus, ESCROW_SEED, LISTING_SEED},
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = RentalInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        RentalInstruction::CreateListing {
            listing_id,
            deposit_lamports,
            rental_fee_lamports,
            max_duration_slots,
        } => create_listing(
            program_id,
            accounts,
            listing_id,
            deposit_lamports,
            rental_fee_lamports,
            max_duration_slots,
        ),
        RentalInstruction::BookRental {
            start_slot,
            end_slot,
        } => book_rental(program_id, accounts, start_slot, end_slot),
        RentalInstruction::CompleteRental => complete_rental(accounts),
        RentalInstruction::CancelListing => cancel_listing(accounts),
        RentalInstruction::FlagDispute => flag_dispute(accounts),
    }
}

fn create_listing(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    listing_id: u64,
    deposit_lamports: u64,
    rental_fee_lamports: u64,
    max_duration_slots: u64,
) -> ProgramResult {
    if deposit_lamports == 0 || max_duration_slots == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let account_info_iter = &mut accounts.iter();
    let owner = next_account_info(account_info_iter)?;
    let listing_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (expected_listing, bump) = listing_pda(program_id, owner.key, listing_id);
    if expected_listing != *listing_account.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let rent_lamports = Rent::get()?.minimum_balance(Listing::LEN);
    invoke_signed(
        &system_instruction::create_account(
            owner.key,
            listing_account.key,
            rent_lamports,
            Listing::LEN as u64,
            program_id,
        ),
        &[
            owner.clone(),
            listing_account.clone(),
            system_program.clone(),
        ],
        &[&[
            LISTING_SEED,
            owner.key.as_ref(),
            &listing_id.to_le_bytes(),
            &[bump],
        ]],
    )?;

    let (_, escrow_bump) = escrow_pda(program_id, listing_account.key);
    let listing = Listing {
        owner: *owner.key,
        renter: Pubkey::default(),
        listing_id,
        deposit_lamports,
        rental_fee_lamports,
        max_duration_slots,
        start_slot: 0,
        end_slot: 0,
        bump,
        escrow_bump,
        status: RentalStatus::Listed,
    };
    listing.pack(&mut listing_account.data.borrow_mut())?;
    msg!("listing created");
    Ok(())
}

fn book_rental(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    start_slot: u64,
    end_slot: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let renter = next_account_info(account_info_iter)?;
    let owner = next_account_info(account_info_iter)?;
    let listing_account = next_account_info(account_info_iter)?;
    let escrow_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if !renter.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut listing = Listing::unpack(&listing_account.data.borrow())?;
    if listing.status != RentalStatus::Listed || listing.owner != *owner.key {
        return Err(ProgramError::InvalidAccountData);
    }
    if start_slot >= end_slot || end_slot - start_slot > listing.max_duration_slots {
        return Err(ProgramError::InvalidInstructionData);
    }

    let current_slot = Clock::get()?.slot;
    if end_slot <= current_slot {
        return Err(ProgramError::InvalidInstructionData);
    }

    let (expected_escrow, escrow_bump) = escrow_pda(program_id, listing_account.key);
    if expected_escrow != *escrow_account.key || escrow_bump != listing.escrow_bump {
        return Err(ProgramError::InvalidSeeds);
    }

    let escrow_rent = Rent::get()?.minimum_balance(0);
    invoke_signed(
        &system_instruction::create_account(
            renter.key,
            escrow_account.key,
            listing
                .deposit_lamports
                .checked_add(escrow_rent)
                .ok_or(ProgramError::InvalidInstructionData)?,
            0,
            program_id,
        ),
        &[
            renter.clone(),
            escrow_account.clone(),
            system_program.clone(),
        ],
        &[&[
            ESCROW_SEED,
            listing_account.key.as_ref(),
            &[listing.escrow_bump],
        ]],
    )?;

    if listing.rental_fee_lamports > 0 {
        invoke(
            &system_instruction::transfer(renter.key, owner.key, listing.rental_fee_lamports),
            &[renter.clone(), owner.clone(), system_program.clone()],
        )?;
    }

    listing.renter = *renter.key;
    listing.start_slot = start_slot;
    listing.end_slot = end_slot;
    listing.status = RentalStatus::Booked;
    listing.pack(&mut listing_account.data.borrow_mut())?;
    msg!("rental booked");
    Ok(())
}

fn complete_rental(accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let owner = next_account_info(account_info_iter)?;
    let renter = next_account_info(account_info_iter)?;
    let listing_account = next_account_info(account_info_iter)?;
    let escrow_account = next_account_info(account_info_iter)?;

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut listing = Listing::unpack(&listing_account.data.borrow())?;
    if listing.status != RentalStatus::Booked
        || listing.owner != *owner.key
        || listing.renter != *renter.key
    {
        return Err(ProgramError::InvalidAccountData);
    }

    drain_account(escrow_account, renter)?;
    listing.status = RentalStatus::Completed;
    listing.pack(&mut listing_account.data.borrow_mut())?;
    msg!("deposit returned");
    Ok(())
}

fn cancel_listing(accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let owner = next_account_info(account_info_iter)?;
    let listing_account = next_account_info(account_info_iter)?;

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut listing = Listing::unpack(&listing_account.data.borrow())?;
    if listing.status != RentalStatus::Listed || listing.owner != *owner.key {
        return Err(ProgramError::InvalidAccountData);
    }

    listing.status = RentalStatus::Cancelled;
    listing.pack(&mut listing_account.data.borrow_mut())?;
    drain_account(listing_account, owner)?;
    msg!("listing cancelled");
    Ok(())
}

fn flag_dispute(accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let signer = next_account_info(account_info_iter)?;
    let listing_account = next_account_info(account_info_iter)?;

    if !signer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut listing = Listing::unpack(&listing_account.data.borrow())?;
    if listing.status != RentalStatus::Booked {
        return Err(ProgramError::InvalidAccountData);
    }
    if *signer.key != listing.owner && *signer.key != listing.renter {
        return Err(ProgramError::IllegalOwner);
    }

    listing.status = RentalStatus::Disputed;
    listing.pack(&mut listing_account.data.borrow_mut())?;
    msg!("rental disputed");
    Ok(())
}

fn drain_account(from: &AccountInfo, to: &AccountInfo) -> ProgramResult {
    let lamports = **from.lamports.borrow();
    **to.lamports.borrow_mut() = to
        .lamports()
        .checked_add(lamports)
        .ok_or(ProgramError::InvalidAccountData)?;
    **from.lamports.borrow_mut() = 0;
    Ok(())
}
