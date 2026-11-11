#![no_std]

use pinocchio::{
    AccountView,
    Address,
    ProgramResult,
    error::ProgramError,
    program_entrypoint,
    default_allocator,
    nostd_panic_handler,
};

program_entrypoint!(process_instruction);
default_allocator!();
nostd_panic_handler!();

// Instruction identifiers
const INITIALIZE: u8 = 0;
const SET_LED: u8 = 1;

// Account layout
const ACCOUNT_DISCRIMINATOR: &[u8; 8] = b"LEDSTATE";
const LED_STATE_LEN: usize = 41;

pub fn process_instruction(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {

    // Identify which instruction the client is requesting.
    if instruction_data.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }

    match instruction_data[0] {

        // Instruction 0: Initialize the LED account
        INITIALIZE => {
            if instruction_data.len() != 1 {
                return Err(ProgramError::InvalidInstructionData);
            }

            initialize(program_id, accounts)
        }

        // Instruction 1: Turn LED ON or OFF
        SET_LED => {
            if instruction_data.len() != 2 {
                return Err(ProgramError::InvalidInstructionData);
            }

            let state = instruction_data[1];

            if state != 0 && state != 1 {
                return Err(ProgramError::InvalidInstructionData);
            }

            set_led(program_id, accounts, state)
        }

        _ => Err(ProgramError::InvalidInstructionData),
    }
}

// INITIALIZE ACCOUNT
fn initialize(
    program_id: &Address,
    accounts: &mut [AccountView],
) -> ProgramResult {

    let [state_account, authority] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Authority must sign the transaction.
    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // State account must be writable and owned by our program.
    if !state_account.is_writable()
        || !state_account.owned_by(program_id)
        || state_account.data_len() != LED_STATE_LEN
    {
        return Err(ProgramError::InvalidAccountData);
    }

    let mut data = state_account.try_borrow_mut()?;

    // Only allow initialization of a completely zeroed account.
    if data.iter().any(|byte| *byte != 0) {
        return Err(ProgramError::InvalidAccountData);
    }

    // Write account identifier.
    data[0..8].copy_from_slice(ACCOUNT_DISCRIMINATOR);

    // Store the authority's public key.
    data[8..40].copy_from_slice(authority.address().as_ref());

    // Initial LED state = OFF.
    data[40] = 0;

    Ok(())
}

// SET LED STATE
fn set_led(
    program_id: &Address,
    accounts: &mut [AccountView],
    state: u8,
) -> ProgramResult {

    let [state_account, authority] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !state_account.is_writable()
        || !state_account.owned_by(program_id)
        || state_account.data_len() != LED_STATE_LEN
    {
        return Err(ProgramError::InvalidAccountData);
    }

    // Check account identifier and authority.
    {
        let data = state_account.try_borrow()?;

        if data[0..8] != ACCOUNT_DISCRIMINATOR[..] {
            return Err(ProgramError::InvalidAccountData);
        }

        if data[8..40] != authority.address().as_ref()[..] {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    // Update LED state.
    let mut data = state_account.try_borrow_mut()?;

    data[40] = state;

    Ok(())
}