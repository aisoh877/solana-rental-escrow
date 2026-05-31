use anyhow::{anyhow, Context, Result};
use borsh::BorshDeserialize;
use clap::{Parser, Subcommand};
use rental_escrow_program::{
    instruction,
    state::{Listing, RentalStatus},
};
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::LAMPORTS_PER_SOL,
    signature::{read_keypair_file, Keypair, Signer},
    transaction::Transaction,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rental-escrow")]
#[command(about = "CLI for the P2P rental escrow Solana program")]
struct Cli {
    #[arg(long, default_value = "https://api.devnet.solana.com")]
    rpc_url: String,
    #[arg(long)]
    program_id: Pubkey,
    #[arg(long, default_value = "~/.config/solana/id.json")]
    keypair: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Create {
        #[arg(long)]
        listing_id: u64,
        #[arg(long)]
        deposit_sol: f64,
        #[arg(long)]
        fee_sol: f64,
        #[arg(long)]
        max_duration_slots: u64,
    },
    Book {
        #[arg(long)]
        owner: Pubkey,
        #[arg(long)]
        listing_id: u64,
        #[arg(long)]
        start_slot: u64,
        #[arg(long)]
        end_slot: u64,
    },
    Complete {
        #[arg(long)]
        owner: Pubkey,
        #[arg(long)]
        renter: Pubkey,
        #[arg(long)]
        listing_id: u64,
    },
    Cancel {
        #[arg(long)]
        listing_id: u64,
    },
    Dispute {
        #[arg(long)]
        owner: Pubkey,
        #[arg(long)]
        listing_id: u64,
    },
    Show {
        #[arg(long)]
        owner: Pubkey,
        #[arg(long)]
        listing_id: u64,
    },
    Pda {
        #[arg(long)]
        owner: Pubkey,
        #[arg(long)]
        listing_id: u64,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = RpcClient::new_with_commitment(cli.rpc_url, CommitmentConfig::confirmed());
    let payer = read_keypair(&cli.keypair)?;

    match cli.command {
        Command::Create {
            listing_id,
            deposit_sol,
            fee_sol,
            max_duration_slots,
        } => {
            let ix = instruction::create_listing(
                cli.program_id,
                payer.pubkey(),
                listing_id,
                sol_to_lamports(deposit_sol)?,
                sol_to_lamports(fee_sol)?,
                max_duration_slots,
            );
            send(&client, &payer, vec![ix])?;
            print_pdas(cli.program_id, payer.pubkey(), listing_id);
        }
        Command::Book {
            owner,
            listing_id,
            start_slot,
            end_slot,
        } => {
            let ix = instruction::book_rental(
                cli.program_id,
                owner,
                payer.pubkey(),
                listing_id,
                start_slot,
                end_slot,
            );
            send(&client, &payer, vec![ix])?;
            print_pdas(cli.program_id, owner, listing_id);
        }
        Command::Complete {
            owner,
            renter,
            listing_id,
        } => {
            let ix = instruction::complete_rental(cli.program_id, owner, renter, listing_id);
            send(&client, &payer, vec![ix])?;
        }
        Command::Cancel { listing_id } => {
            let ix = instruction::cancel_listing(cli.program_id, payer.pubkey(), listing_id);
            send(&client, &payer, vec![ix])?;
        }
        Command::Dispute { owner, listing_id } => {
            let ix = instruction::flag_dispute(cli.program_id, payer.pubkey(), owner, listing_id);
            send(&client, &payer, vec![ix])?;
        }
        Command::Show { owner, listing_id } => {
            let (listing, _) = instruction::listing_pda(&cli.program_id, &owner, listing_id);
            let account = client
                .get_account(&listing)
                .with_context(|| format!("fetch listing account {listing}"))?;
            let listing_state = Listing::try_from_slice(&account.data)?;
            print_listing(&listing_state);
        }
        Command::Pda { owner, listing_id } => {
            print_pdas(cli.program_id, owner, listing_id);
        }
    }

    Ok(())
}

fn read_keypair(path: &str) -> Result<Keypair> {
    let expanded = if let Some(rest) = path.strip_prefix("~/") {
        let home = std::env::var("HOME").context("HOME is not set")?;
        PathBuf::from(home).join(rest)
    } else {
        PathBuf::from(path)
    };
    read_keypair_file(&expanded)
        .map_err(|err| anyhow!("read keypair {}: {err}", expanded.display()))
}

fn send(
    client: &RpcClient,
    payer: &Keypair,
    instructions: Vec<solana_sdk::instruction::Instruction>,
) -> Result<()> {
    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer],
        blockhash,
    );
    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("signature: {sig}");
    println!("explorer: https://explorer.solana.com/tx/{sig}?cluster=devnet");
    Ok(())
}

fn sol_to_lamports(sol: f64) -> Result<u64> {
    if !sol.is_finite() || sol < 0.0 {
        return Err(anyhow!("SOL amount must be non-negative"));
    }
    Ok((sol * LAMPORTS_PER_SOL as f64).round() as u64)
}

fn print_pdas(program_id: Pubkey, owner: Pubkey, listing_id: u64) {
    let (listing, _) = instruction::listing_pda(&program_id, &owner, listing_id);
    let (escrow, _) = instruction::escrow_pda(&program_id, &listing);
    println!("listing: {listing}");
    println!("escrow:  {escrow}");
}

fn print_listing(listing: &Listing) {
    println!("owner: {}", listing.owner);
    println!("renter: {}", listing.renter);
    println!("listing_id: {}", listing.listing_id);
    println!("deposit_lamports: {}", listing.deposit_lamports);
    println!("rental_fee_lamports: {}", listing.rental_fee_lamports);
    println!("max_duration_slots: {}", listing.max_duration_slots);
    println!("start_slot: {}", listing.start_slot);
    println!("end_slot: {}", listing.end_slot);
    println!("status: {}", status_name(listing.status));
}

fn status_name(status: RentalStatus) -> &'static str {
    match status {
        RentalStatus::Listed => "listed",
        RentalStatus::Booked => "booked",
        RentalStatus::Completed => "completed",
        RentalStatus::Cancelled => "cancelled",
        RentalStatus::Disputed => "disputed",
    }
}
