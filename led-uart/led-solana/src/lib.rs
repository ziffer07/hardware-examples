#![no_std]

use pinocchio::{
    AccountView,
    Address,
    entrypoint,
    ProgramResult,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {

    Ok(())
}