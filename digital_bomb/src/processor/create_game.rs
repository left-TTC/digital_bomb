use web3_utils::{
    BorshSize, InstructionsAccount, accounts::InstructionsAccount, borsh_size::BorshSize, check::{check_account_key, check_signer}
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info}, 
    entrypoint::ProgramResult, msg, program::invoke_signed, 
    program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, 
};

use solana_system_interface::instruction as system_instruction;

use crate::{constants::SYSTEM_ID, state::game::{GameLevel, GameRecord}, utils::get_seeds_and_key};


#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
pub struct Params {
    // means the limitation of the game's answer
    pub max_number: u16,
    // the odds * 100
    pub odds_x100: u32,
    // the calculated splicing hash value
    pub splicing_hash: [u8; 32],
    // game level -- Determine how much SOL to bet
    pub game_level: u8,
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

pub fn process_create_game<'a, 'b: 'a>(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {

    let accounts = Accounts::parse(accounts)?;

    let game_record = accounts.game_recorder;
    let (game, game_seeds) = 
        get_seeds_and_key(params.splicing_hash.to_vec());
    check_account_key(game_record, &game)?;

    let pre_store: u64 = GameLevel::from_u8(&params.game_level)?.get_bet() *
        std::cmp::max(params.odds_x100, (params.max_number * 100) as u32) as u64;

    invoke_signed(
        &system_instruction::create_account(
            accounts.promoter.key, 
            game_record.key, 
            pre_store, 
            GameRecord::LEN as u64, 
            &crate::ID
        ),
        &[
            accounts.promoter.clone(),
            accounts.game_recorder.clone(),
            accounts.system_program.clone(),
        ],
        &[&game_seeds.chunks(32).collect::<Vec<&[u8]>>()],
    )?;
    msg!("create game record account ok");

    let record_init = GameRecord::new(
        *accounts.promoter.key, 
        params.max_number, 
        params.odds_x100,
        params.game_level,
    );
    let mut data = accounts.game_recorder.try_borrow_mut_data()?;
    record_init.pack_into_slice(&mut data);
    msg!("init game record ok");
    
    Ok(())
}