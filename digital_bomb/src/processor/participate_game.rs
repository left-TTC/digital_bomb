use web3_utils::{
    BorshSize, InstructionsAccount, accounts::InstructionsAccount, borsh_size::BorshSize, check::{check_account_key, check_signer}
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info}, clock::Clock, entrypoint::ProgramResult, msg, program::{invoke}, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, sysvar::Sysvar
};

use solana_system_interface::instruction as system_instruction;

use crate::{constants::SYSTEM_ID, state::game::GameRecord};


#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
pub struct Params {
    pub point: u16,
}


#[derive(InstructionsAccount)]
/// The required accounts for the `create` instruction
pub struct Accounts<'a, T> { 
    #[cons(writable, signer)]
    pub player: &'a T,
    #[cons(writable)]
    pub game_recorder: &'a T,
    /// The system program account
    pub system_program: &'a T,
    #[cons(writable)]
    /// The promoter
    pub promoter: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        Ok(Accounts {
            player:next_account_info(accounts_iter)?,
            game_recorder: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            promoter: next_account_info(accounts_iter)?,
        })
    }

    pub fn check(&self) -> ProgramResult {
        check_account_key(self.system_program, &SYSTEM_ID).unwrap();
        check_signer(self.player).unwrap();
        msg!("promoter ok");

        Ok(())
    }
}

pub fn process_participate_game<'a, 'b: 'a>(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {

    let accounts = Accounts::parse(accounts)?;

    let promoter = accounts.promoter;
    let game_record = accounts.game_recorder;
    
    let mut data = {
        let data_ref = game_record.try_borrow_data()?;    
        GameRecord::unpack_from_slice(&data_ref)?        
    }; 

    if params.point == 0 || params.point > data.max {
        msg!("x too large or x = 0");
        return Err(ProgramError::InvalidArgument);
    }

    if &data.promoter != promoter.key {
        msg!("give an fault promoter");
        return Err(ProgramError::InvalidArgument);
    }

    invoke(
        &system_instruction::transfer(
            accounts.player.key, 
            promoter.key, 
            data.level.get_bet()
        ), 
        &[
            accounts.player.clone(),
            accounts.promoter.clone(),
            accounts.system_program.clone(),
        ]
    )?;
    msg!("transfer bet ok");

    data.player = *promoter.key;
    data.firing_point = params.point;
    data.shot_time = Clock::get()?.unix_timestamp;

    {
        let mut data_mut = game_record.try_borrow_mut_data()?;
        data.pack_into_slice(&mut data_mut);   
        msg!("write game record ok");         
    }
    
    Ok(())
}