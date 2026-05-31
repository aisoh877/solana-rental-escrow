pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint {
    use crate::processor::process_instruction;
    use solana_program::entrypoint;

    entrypoint!(process_instruction);
}

solana_program::declare_id!("Bo556kwX6HB4RKteyPsRY8F7SxQjzJanLSFNWdbiF8j3");
