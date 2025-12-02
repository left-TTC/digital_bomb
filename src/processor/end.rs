use web3_utils::{
    InstructionsAccount, accounts::InstructionsAccount, check::{check_account_key, check_signer}
};
use solana_program::{
    account_info::{AccountInfo, next_account_info}, 
    entrypoint::ProgramResult, msg,
    program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, 
};


use crate::{constants::{VAULT}, state::game::GameRecord, utils::{if_reveal_time, percent_of}};

#[derive(InstructionsAccount)]
/// The required accounts for the `create` instruction
pub struct Accounts<'a, T> { 
    #[cons(writable, signer)]
    pub terminator: &'a T,
    #[cons(writable)]
    pub promoter: &'a T,
    #[cons(writable)]
    pub player: &'a T,
    #[cons(writable)]
    pub game_recorder: &'a T,
    #[cons(writable)]
    pub vault: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        Ok(Accounts {
            terminator:next_account_info(accounts_iter)?,
            promoter: next_account_info(accounts_iter)?,
            player: next_account_info(accounts_iter)?,
            game_recorder: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
        })
    }

    pub fn check(&self) -> ProgramResult {
        check_account_key(self.vault, &VAULT).unwrap();

        check_signer(self.terminator).unwrap();
        msg!("promoter ok");

        Ok(())
    }
}

pub fn process_end_game<'a, 'b: 'a>(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {

    let accounts = Accounts::parse(accounts)?;

    let game_record = accounts.game_recorder;
    let player = accounts.player;
    let promoter = accounts.promoter;

    let data = {
        let data_ref = game_record.try_borrow_data()?;    
        GameRecord::unpack_from_slice(&data_ref)?        
    }; 

    if data.firing_point == 0 || data.player == crate::ID {
        msg!("the game hasn't started");
        return Err(ProgramError::InvalidArgument);
    } 

    let mut if_pay_more = match (if_reveal_time(data.shot_time)?, data.answer) {
        (false, 0) => {
            msg!("over time and no reveal");
            true
        }
        (false, _) => {
            msg!("over time and revealed");
            false
        }
        (true, 0) => {
            msg!("Settlement cannot be made before reveal");
            return Err(ProgramError::InvalidArgument);
        }
        (true, _) => {
            msg!("not over time but revealed");
            false
        },
    };

    if data.answer == 0 {
        msg!("can't set 0 as the anwser");
        if_pay_more = true
    }

    if &data.promoter != promoter.key || &data.player != player.key {
        msg!("give fault promoter or player");
        return Err(ProgramError::InvalidArgument);
    }

    let terminator = accounts.terminator;

    if terminator.key != &data.player && terminator.key != &data.promoter {
        msg!("Incorrect Settler");
        return Err(ProgramError::InvalidArgument);
    }

    let all_lamports = accounts.game_recorder.lamports();
    let win: u64 = data.level.get_bet() * data.odds_x100 as u64 / 100;

    if data.firing_point == data.answer {
        msg!("player win");

        let vault_fee = percent_of(win, 1);
        let player_gain = win.checked_sub(vault_fee).unwrap();
        let promoter_back = all_lamports.checked_sub(win).unwrap();
        
        **accounts.game_recorder.try_borrow_mut_lamports()? -= all_lamports;
        **accounts.player.try_borrow_mut_lamports()? += player_gain;
        **accounts.promoter.try_borrow_mut_lamports()? += promoter_back;
        **accounts.vault.try_borrow_mut_lamports()? += vault_fee;
    }else {
        msg!("promoter win");

        let vault_fee = percent_of(win, 1);

        **accounts.game_recorder.try_borrow_mut_lamports()? -= all_lamports;
        if if_pay_more {
            msg!("promoter didn't reveal the anwser");
            **accounts.player.try_borrow_mut_lamports()? += all_lamports.checked_sub(vault_fee).unwrap();
        }else {
            **accounts.promoter.try_borrow_mut_lamports()? += all_lamports.checked_sub(vault_fee).unwrap();
        }
        **accounts.vault.try_borrow_mut_lamports()? += vault_fee;
    }

    let mut game_recorder_data = accounts.game_recorder.try_borrow_mut_data()?;
    game_recorder_data.fill(0);
    
    Ok(())
}