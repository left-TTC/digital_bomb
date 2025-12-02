use solana_program::clock::Clock;
use solana_program::hash::Hash;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;
use solana_program::{hash::hashv};

use crate::constants::REVEAL_TIME;


pub fn get_seeds_and_key(
    hashed_value: Vec<u8>, 
) -> (Pubkey, Vec<u8>) {
    let mut seeds_vec: Vec<u8> = hashed_value;

    let hash_byte = hashv(&[("game").as_bytes()])
        .as_ref()
        .to_vec();

    for b in hash_byte {
        seeds_vec.push(b);
    }

    let (pda, bump) =
        Pubkey::find_program_address(&seeds_vec.chunks(32).collect::<Vec<&[u8]>>(), &crate::ID);
    seeds_vec.push(bump);

    (pda, seeds_vec)
}

pub fn get_splicing_hash(
    x: u16,
    random: [u8; 6],
) -> Vec<u8> {

    let x_bytes = x.to_le_bytes();
    let x_hash: Hash = hashv(&[&x_bytes]);

    let random_hash: Hash = hashv(&[&random]);

    let combined_hash: Hash = hashv(&[x_hash.as_ref(), random_hash.as_ref()]);

    combined_hash.as_ref().to_vec()
}

pub fn if_reveal_time(
    record_time: i64
) -> Result<bool, ProgramError> {
    let now = Clock::get()?.unix_timestamp;

    if record_time + REVEAL_TIME >= now {
        Ok(true)
    }else {
        Ok(false)
    }
}

pub fn percent_of(value: u64, percent: u64) -> u64 {
    value.saturating_mul(percent).saturating_div(100)
}