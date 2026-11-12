use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_system_interface::instruction as system_instruction;

use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer, read_keypair_file, write_keypair_file},
    transaction::Transaction,
};

use std::{
    env,
    io::{self, Write},
    path::Path,
    str::FromStr,
};

const ACCOUNT_SIZE: usize = 41;
const STATE_KEYPAIR_FILE: &str = "led-state-keypair.json";

fn main() -> Result<()> {

    // 1. Connect to Devnet
    let rpc = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );

    // 2. Get Program ID from command-line argument
    let args: Vec<String> = env::args().collect();

    let program_id = Pubkey::from_str(
        args.get(1).context("Usage: cargo run -- <PROGRAM_ID>")?
    )?;

    // 3. Load your existing Solana wallet
    let home = env::var("HOME")?;

    let wallet_path = format!(
        "{}/.config/solana/id.json",
        home
    );

    let payer = read_keypair_file(wallet_path)
        .map_err(|e| anyhow::anyhow!("Could not load wallet: {}", e))?;

    println!("Wallet: {}", payer.pubkey());
    println!("Program: {}", program_id);

    // 4. Load or generate the LED state account keypair
    let state_account = if Path::new(STATE_KEYPAIR_FILE).exists() {
        println!("Loading existing LED account...");
        read_keypair_file(STATE_KEYPAIR_FILE)
            .map_err(|e| anyhow::anyhow!("Could not load LED keypair: {}", e))?
    } else {
        println!("Generating new LED account...");
        let keypair = Keypair::new();

        write_keypair_file(&keypair, STATE_KEYPAIR_FILE)
            .map_err(|e| anyhow::anyhow!("Could not save LED keypair: {}", e))?;

        keypair
    };

    println!("LED account: {}", state_account.pubkey());

    // 5. Check whether the account already exists
    if rpc.get_account(&state_account.pubkey()).is_err() {

        println!("\nCreating LED account...");

        let rent = rpc.get_minimum_balance_for_rent_exemption(
            ACCOUNT_SIZE
        )?;

        // Create account and assign ownership to our program
        let create_instruction = system_instruction::create_account(
            &payer.pubkey(),
            &state_account.pubkey(),
            rent,
            ACCOUNT_SIZE as u64,
            &program_id,
        );

        // Initialize instruction: [0]
        let initialize_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(state_account.pubkey(), false),
                AccountMeta::new_readonly(payer.pubkey(), true),
            ],
            data: vec![0],
        };

        // Both instructions go into one transaction
        let instructions = vec![
            create_instruction,
            initialize_instruction,
        ];

        let blockhash = rpc.get_latest_blockhash()?;

        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[&payer, &state_account],
            blockhash,
        );

        let signature = rpc.send_and_confirm_transaction(&transaction)?;

        println!("Account initialized!");
        println!("Transaction: {}", signature);

    } else {
        println!("LED account already exists.");
    }

    // 6. Interactive LED control
    println!("\n==============================");
    println!("       SOLANA LED CONTROL");
    println!("==============================");
    println!("Enter 1 = LED ON");
    println!("Enter 0 = LED OFF");
    println!("Enter q = Quit");

    loop {

        print!("\nCommand: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let command = input.trim();

        if command == "q" {
            break;
        }

        let state: u8 = match command {
            "1" => 1,
            "0" => 0,
            _ => {
                println!("Invalid command.");
                continue;
            }
        };

        // Build the SetLed instruction: [1, state]
        let instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(state_account.pubkey(), false),
                AccountMeta::new_readonly(payer.pubkey(), true),
            ],
            data: vec![1, state],
        };

        let blockhash = rpc.get_latest_blockhash()?;

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer],
            blockhash,
        );

        // Send transaction and wait for confirmation
        let signature = rpc.send_and_confirm_transaction(&transaction)?;

        println!("Transaction confirmed: {}", signature);

        // Read the account back from Solana
        let account = rpc.get_account(&state_account.pubkey())?;

        let actual_state = account.data[40];

        if actual_state == 1 {
            println!("Solana LED State: ON");
        } else {
            println!("Solana LED State: OFF");
        }
    }

    println!("Exiting LED controller.");

    Ok(())
}