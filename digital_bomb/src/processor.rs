use crate::instruction::ProgramInstruction;
use borsh::BorshDeserialize;
use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

pub struct Processor {}

pub mod create_game;
pub mod end;
pub mod participate_game;
pub mod reveal;
pub mod delete;

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Beginning processing");
        msg!("instruction: {:?}", instruction_data);

        let instruction = FromPrimitive::from_u8(instruction_data[0])
            .ok_or(ProgramError::InvalidInstructionData)?;
        let instruction_data = &instruction_data[1..];

        msg!("Instruction unpacked: means instruction data is ok");

        match instruction {
            ProgramInstruction::CreateGame => {
                msg!("Instruction: start a game");
                let params = create_game::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_game::process_create_game(program_id, accounts, params)?;
            }
            ProgramInstruction::Participate => {
                msg!("Instruction: Participate in a game");
                let params = participate_game::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                participate_game::process_participate_game(program_id, accounts, params)?;
            }
            ProgramInstruction::Reveal => {
                msg!("Instruction: reveal a game");
                let params = reveal::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                reveal::process_reveal_game(program_id, accounts, params)?;
            }
            ProgramInstruction::End => {
                msg!("Instruction: Finish a game");
                end::process_end_game(program_id, accounts)?;
            }
            ProgramInstruction::Delete => {
                delete::process_delete_game(program_id, accounts)?;
            }
        }

        Ok(())
    }
}
