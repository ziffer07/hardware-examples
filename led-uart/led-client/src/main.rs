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
    thread,
    time::Duration,
};

const ACCOUNT_SIZE: usize = 41;
const STATE_KEYPAIR_FILE: &str = "led-state-keypair.json";
const SERIAL_BAUD_RATE: u32 = 9600;
const READY_REQUEST: u8 = b'?';
const READY_RESPONSE: u8 = b'R';
const SERIAL_RETRIES: usize = 20;

fn main() -> Result<()> {

    // 1. Connect to Devnet
    let rpc = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );

    // 2. Get Program ID from command-line argument
    let args: Vec<String> = env::args().collect();

    let program_id = Pubkey::from_str(
        args.get(1)
            .context("Usage: cargo run -- <PROGRAM_ID> <SERIAL_PORT>")?
    )?;
    let serial_path = args
        .get(2)
        .context("Usage: cargo run -- <PROGRAM_ID> <SERIAL_PORT>")?;

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

    // 5. Connect to the Arduino and wait for its ready response.
    let mut serial_port = serialport::new(serial_path, SERIAL_BAUD_RATE)
        .timeout(Duration::from_millis(100))
        .open()
        .with_context(|| format!("Could not open serial port {serial_path}"))?;

    wait_for_firmware(&mut *serial_port)?;
    println!("Arduino ready on {serial_path} at {SERIAL_BAUD_RATE} baud.");

    // 6. Check whether the account already exists
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

    // 7. Interactive LED control
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

        if account.owner != program_id {
            anyhow::bail!(
                "LED state account owner mismatch: expected {program_id}, got {}",
                account.owner
            );
        }
        if account.data.len() != ACCOUNT_SIZE {
            anyhow::bail!(
                "LED state account has an unexpected size: expected {ACCOUNT_SIZE}, got {}",
                account.data.len()
            );
        }

        let actual_state = account.data[40];
        if actual_state != state {
            anyhow::bail!(
                "Transaction was confirmed, but on-chain LED state is {actual_state}, expected {state}"
            );
        }

        if actual_state == 1 {
            println!("Solana LED State: ON");
        } else {
            println!("Solana LED State: OFF");
        }

        send_led_command(&mut *serial_port, state).with_context(|| {
            format!(
                "On-chain state is {actual_state}, but the Arduino did not acknowledge the command"
            )
        })?;
        println!("Arduino acknowledged LED state: {actual_state}");
    }

    println!("Exiting LED controller.");

    Ok(())
}

fn wait_for_firmware(port: &mut dyn serialport::SerialPort) -> Result<()> {
    for _ in 0..SERIAL_RETRIES {
        port.write_all(&[READY_REQUEST])
            .context("Could not send readiness request to Arduino")?;
        port.flush()
            .context("Could not flush readiness request to Arduino")?;
        thread::sleep(Duration::from_millis(250));

        if port
            .bytes_to_read()
            .context("Could not check for Arduino response")?
            > 0
        {
            let mut response = [0];
            port.read_exact(&mut response)
                .context("Could not read Arduino readiness response")?;
            if response[0] == READY_RESPONSE {
                return Ok(());
            }
        }
    }

    anyhow::bail!(
        "Arduino did not become ready on the serial port; check the port and ensure the Rust firmware is running"
    )
}

fn send_led_command(port: &mut dyn serialport::SerialPort, state: u8) -> Result<()> {
    let command = b'0' + state;
    port.write_all(&[command])
        .context("Could not send LED command to Arduino")?;
    port.flush()
        .context("Could not flush LED command to Arduino")?;

    for _ in 0..SERIAL_RETRIES {
        thread::sleep(Duration::from_millis(50));
        if port
            .bytes_to_read()
            .context("Could not check for Arduino acknowledgment")?
            > 0
        {
            let mut response = [0];
            port.read_exact(&mut response)
                .context("Could not read Arduino acknowledgment")?;
            if response[0] == command {
                return Ok(());
            }
            anyhow::bail!(
                "Arduino acknowledged unexpected command byte {:?}",
                response[0] as char
            );
        }
    }

    anyhow::bail!(
        "Timed out waiting for Arduino acknowledgment; the LED may have changed"
    )
}