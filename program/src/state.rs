use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

pub const LISTING_SEED: &[u8] = b"listing";
pub const ESCROW_SEED: &[u8] = b"escrow";

#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum RentalStatus {
    Listed,
    Booked,
    Completed,
    Cancelled,
    Disputed,
}

impl RentalStatus {
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Listed => 0,
            Self::Booked => 1,
            Self::Completed => 2,
            Self::Cancelled => 3,
            Self::Disputed => 4,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub struct Listing {
    pub owner: Pubkey,
    pub renter: Pubkey,
    pub listing_id: u64,
    pub deposit_lamports: u64,
    pub rental_fee_lamports: u64,
    pub max_duration_slots: u64,
    pub start_slot: u64,
    pub end_slot: u64,
    pub bump: u8,
    pub escrow_bump: u8,
    pub status: RentalStatus,
}

impl Listing {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 1 + 1;

    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        Self::try_from_slice(data).map_err(|_| ProgramError::InvalidAccountData)
    }

    pub fn pack(&self, data: &mut [u8]) -> Result<(), ProgramError> {
        self.serialize(&mut &mut data[..])
            .map_err(|_| ProgramError::InvalidAccountData)
    }
}
