use web3_utils::{
    BorshSize, InstructionsAccount, accounts::InstructionsAccount, borsh_size::BorshSize, check::{check_account_key, check_signer}
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info}, 
    entrypoint::ProgramResult, msg,
    program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, 
};

use crate::{constants::SYSTEM_ID, state::game::GameRecord, utils::{get_seeds_and_key, get_splicing_hash, if_reveal_time}};


#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
pub struct Params {
    pub x: u16,
    pub random: [u8; 6],
}


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

pub fn process_reveal_game<'a, 'b: 'a>(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {

    let accounts = Accounts::parse(accounts)?;

    let game_record = accounts.game_recorder;
    let (game, _) = 
        get_seeds_and_key(get_splicing_hash(params.x, params.random));
    check_account_key(game_record, &game)?;
    msg!("gived x and random is correct");

    let mut data = {
        let data_ref = game_record.try_borrow_data()?;    
        GameRecord::unpack_from_slice(&data_ref)?        
    }; 

    let promoter = accounts.promoter;

    if &data.promoter != promoter.key {
        msg!("not your game");
        return Err(ProgramError::InvalidArgument);
    }

    if data.firing_point == 0 || data.player == crate::ID {
        msg!("the game hasn't started");
        return Err(ProgramError::InvalidArgument);
    } 

    if !if_reveal_time(data.shot_time)? {
        msg!("over the reveal time");
        return Err(ProgramError::InvalidArgument);
    }

    data.answer = params.x;
    data.random_string = params.random;

    {
        let mut data_mut = game_record.try_borrow_mut_data()?;
        data.pack_into_slice(&mut data_mut);   
        msg!("update game answer ok");         
    }
    
    Ok(())
}