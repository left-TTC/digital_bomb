use web3_utils::{
    InstructionsAccount, accounts::InstructionsAccount, check::{check_account_key, check_signer}
};
use solana_program::{
    account_info::{AccountInfo, next_account_info}, 
    entrypoint::ProgramResult, msg,
    program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, 
};

use crate::{constants::SYSTEM_ID, state::game::GameRecord, };

#[derive(InstructionsAccount)]
/// The required accounts for the `create` instruction
pub struct Accounts<'a, T> { 
    #[cons(writable, signer)]
    pub promoter: &'a T,
    #[cons(writable)]
    pub game_recorder: &'a T,
    /// The system program account
    pub system_program: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        Ok(Accounts {
            promoter:next_account_info(accounts_iter)?,
            game_recorder: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
        })
    }

    pub fn check(&self) -> ProgramResult {
        check_account_key(self.system_program, &SYSTEM_ID).unwrap();

        check_signer(self.promoter).unwrap();
        msg!("promoter ok");

        Ok(())
    }
}

pub fn process_delete_game<'a, 'b: 'a>(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {

    let accounts = Accounts::parse(accounts)?;

    let game_record = accounts.game_recorder;
    let data = {
        let data_ref = game_record.try_borrow_data()?;    
        GameRecord::unpack_from_slice(&data_ref)?        
    }; 

    let promoter = accounts.promoter;

    if &data.promoter != promoter.key {
        msg!("not your game");
        return Err(ProgramError::InvalidArgument);
    }

    if data.firing_point != 0 || data.player == crate::ID {
        msg!("the game has started");
        return Err(ProgramError::InvalidArgument);
    }

    let all_lamports = accounts.game_recorder.lamports();

    {
        **accounts.game_recorder.try_borrow_mut_lamports()? -= all_lamports;
        **accounts.promoter.try_borrow_mut_lamports()? += all_lamports;
        
        msg!("return the pre store ok");
    }

    let mut game_recorder_data = accounts.game_recorder.try_borrow_mut_data()?;
    game_recorder_data.fill(0);
    
    Ok(())
}