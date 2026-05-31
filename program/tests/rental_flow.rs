use borsh::BorshDeserialize;
use rental_escrow_program::{
    id, instruction,
    processor::process_instruction,
    state::{Listing, RentalStatus},
};
use solana_program::clock::Clock;
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

#[tokio::test]
async fn renter_deposit_is_returned_when_owner_completes_rental() {
    let mut context = ProgramTest::new(
        "rental_escrow_program",
        id(),
        processor!(process_instruction),
    )
    .start_with_context()
    .await;

    let owner = clone_keypair(&context.payer);
    let owner_pubkey = owner.pubkey();
    let renter = Keypair::new();
    fund(&mut context, &renter.pubkey(), LAMPORTS_PER_SOL).await;

    let listing_id = 7;
    let deposit = LAMPORTS_PER_SOL / 10;
    let fee = LAMPORTS_PER_SOL / 100;

    send(
        &mut context,
        vec![instruction::create_listing(
            id(),
            owner_pubkey,
            listing_id,
            deposit,
            fee,
            10_000,
        )],
        &[&owner],
    )
    .await;

    let clock: Clock = context.banks_client.get_sysvar().await.unwrap();
    send(
        &mut context,
        vec![instruction::book_rental(
            id(),
            owner_pubkey,
            renter.pubkey(),
            listing_id,
            clock.slot + 1,
            clock.slot + 100,
        )],
        &[&renter],
    )
    .await;

    let listing = fetch_listing(&mut context, owner_pubkey, listing_id).await;
    assert_eq!(listing.status, RentalStatus::Booked);
    assert_eq!(listing.renter, renter.pubkey());

    let before_complete = context
        .banks_client
        .get_balance(renter.pubkey())
        .await
        .unwrap();

    send(
        &mut context,
        vec![instruction::complete_rental(
            id(),
            owner_pubkey,
            renter.pubkey(),
            listing_id,
        )],
        &[&owner],
    )
    .await;

    let after_complete = context
        .banks_client
        .get_balance(renter.pubkey())
        .await
        .unwrap();
    assert!(after_complete > before_complete);

    let listing = fetch_listing(&mut context, owner_pubkey, listing_id).await;
    assert_eq!(listing.status, RentalStatus::Completed);
}

#[tokio::test]
async fn either_party_can_freeze_a_booked_rental_as_disputed() {
    let mut context = ProgramTest::new(
        "rental_escrow_program",
        id(),
        processor!(process_instruction),
    )
    .start_with_context()
    .await;

    let owner = clone_keypair(&context.payer);
    let owner_pubkey = owner.pubkey();
    let renter = Keypair::new();
    fund(&mut context, &renter.pubkey(), LAMPORTS_PER_SOL).await;

    let listing_id = 8;
    send(
        &mut context,
        vec![instruction::create_listing(
            id(),
            owner_pubkey,
            listing_id,
            LAMPORTS_PER_SOL / 20,
            0,
            10_000,
        )],
        &[&owner],
    )
    .await;

    let clock: Clock = context.banks_client.get_sysvar().await.unwrap();
    send(
        &mut context,
        vec![instruction::book_rental(
            id(),
            owner_pubkey,
            renter.pubkey(),
            listing_id,
            clock.slot + 1,
            clock.slot + 20,
        )],
        &[&renter],
    )
    .await;

    send(
        &mut context,
        vec![instruction::flag_dispute(
            id(),
            renter.pubkey(),
            owner_pubkey,
            listing_id,
        )],
        &[&renter],
    )
    .await;

    let listing = fetch_listing(&mut context, owner_pubkey, listing_id).await;
    assert_eq!(listing.status, RentalStatus::Disputed);
}

async fn fund(
    context: &mut solana_program_test::ProgramTestContext,
    recipient: &solana_sdk::pubkey::Pubkey,
    lamports: u64,
) {
    let payer = clone_keypair(&context.payer);
    let payer_pubkey = payer.pubkey();
    send(
        context,
        vec![system_instruction::transfer(
            &payer_pubkey,
            recipient,
            lamports,
        )],
        &[&payer],
    )
    .await;
}

async fn send(
    context: &mut solana_program_test::ProgramTestContext,
    instructions: Vec<solana_sdk::instruction::Instruction>,
    signers: &[&Keypair],
) {
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let payer = signers.first().expect("at least one signer").pubkey();
    let transaction =
        Transaction::new_signed_with_payer(&instructions, Some(&payer), signers, blockhash);
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}

async fn fetch_listing(
    context: &mut solana_program_test::ProgramTestContext,
    owner: solana_sdk::pubkey::Pubkey,
    listing_id: u64,
) -> Listing {
    let (listing, _) = instruction::listing_pda(&id(), &owner, listing_id);
    let account = context
        .banks_client
        .get_account(listing)
        .await
        .unwrap()
        .unwrap();
    Listing::try_from_slice(&account.data).unwrap()
}

fn clone_keypair(keypair: &Keypair) -> Keypair {
    Keypair::from_bytes(&keypair.to_bytes()).unwrap()
}
