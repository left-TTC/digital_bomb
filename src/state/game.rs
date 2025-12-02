use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    program_pack::{Pack, Sealed},
    pubkey::Pubkey,
    msg,
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum GameLevel {
    S, //100 SOL
    A, //10 SOL
    B, //1 SOL
    C, //0.1 SOL
    D, //0.01 SOL
}

impl GameLevel {
    pub fn get_bet(&self) -> u64 {
        match self {
            GameLevel::D => 1_000_000,
            GameLevel::C => 10_000_000,
            GameLevel::B => 100_000_000,
            GameLevel::A => 1_000_000_000,
            GameLevel::S => 10_000_000_000,
        }
    }

    pub fn from_u8(value: &u8) -> Result<Self, ProgramError> {
        match value {
            0 => Ok(GameLevel::S),
            1 => Ok(GameLevel::A),
            2 => Ok(GameLevel::B),
            3 => Ok(GameLevel::C),
            4 => Ok(GameLevel::D),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct GameRecord {
    /// the game promoter
    pub promoter: Pubkey,
    /// the player
    pub player: Pubkey,
    /// the number that player guess
    pub firing_point: u16,
    /// target max value
    pub max: u16,
    /// bet odds x 100
    pub odds_x100: u32,
    /// game's level
    pub level: GameLevel,
    /// timestamp of participation
    pub shot_time: i64,
    /// the correct answer (revealed later)
    pub answer: u16,
    /// the random string
    pub random_string: [u8; 6],
}

impl Sealed for GameRecord {}

impl GameRecord {
    pub fn new(promoter: Pubkey, max: u16, odds: u32, level: u8) -> Self {
        let mut arr = [0u8; 6];
        arr.copy_from_slice("000000".as_bytes());

        let game_level = GameLevel::from_u8(&level).unwrap();

        Self {
            promoter,
            player: crate::ID,     // default program_id 
            firing_point: 0,
            max,
            odds_x100: odds,
            level: game_level,   
            shot_time: 0,
            answer: 0,
            random_string: arr,
        }
    }
    
}

/// total size = 89 bytes
impl Pack for GameRecord {
    const LEN: usize = 32 + 32 + 2 + 2 + 4 + 1 + 8 + 2 + 6;
    
    fn pack_into_slice(&self, dst: &mut [u8]) {
        if dst.len() < Self::LEN {
            msg!("Destination slice too small for GameRecord");
            return;
        }

        let mut offset = 0;

        dst[offset..offset + 32].copy_from_slice(self.promoter.as_ref());
        offset += 32;

        dst[offset..offset + 32].copy_from_slice(self.player.as_ref());
        offset += 32;

        dst[offset..offset + 2].copy_from_slice(&self.firing_point.to_le_bytes());
        offset += 2;

        dst[offset..offset + 2].copy_from_slice(&self.max.to_le_bytes());
        offset += 2;

        dst[offset..offset + 4].copy_from_slice(&self.odds_x100.to_le_bytes());
        offset += 4;

        // level (1 byte)
        dst[offset] = self.level as u8;
        offset += 1;

        dst[offset..offset + 8].copy_from_slice(&self.shot_time.to_le_bytes());
        offset += 8;

        dst[offset..offset + 2].copy_from_slice(&self.answer.to_le_bytes());
        offset += 2;

        dst[offset..offset + 6].copy_from_slice(&self.random_string);
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() < Self::LEN {
            msg!("Source slice too small for GameRecord");
            return Err(ProgramError::InvalidAccountData);
        }

        let mut offset = 0;

        let promoter = Pubkey::new_from_array(src[offset..offset + 32].try_into().unwrap());
        offset += 32;

        let player = Pubkey::new_from_array(src[offset..offset + 32].try_into().unwrap());
        offset += 32;

        let firing_point = u16::from_le_bytes(src[offset..offset + 2].try_into().unwrap());
        offset += 2;

        let max = u16::from_le_bytes(src[offset..offset + 2].try_into().unwrap());
        offset += 2;

        let odds_x100 = u32::from_le_bytes(src[offset..offset + 4].try_into().unwrap());
        offset += 4;

        let level = match src[offset] {
            0 => GameLevel::S,
            1 => GameLevel::A,
            2 => GameLevel::B,
            3 => GameLevel::C,
            4 => GameLevel::D,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        offset += 1;

        let shot_time = i64::from_le_bytes(src[offset..offset + 8].try_into().unwrap());
        offset += 8;

        let answer = u16::from_le_bytes(src[offset..offset + 2].try_into().unwrap());
        offset += 2;

        let random_string = src[offset..offset + 6].try_into().unwrap();

        Ok(Self {
            promoter,
            player,
            firing_point,
            max,
            odds_x100,
            level,
            shot_time,
            answer,
            random_string,
        })
    }
}
