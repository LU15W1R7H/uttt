pub mod generic;
pub mod specific;

use rand::prelude::*;
use serde::{Deserialize, Serialize};

pub const LOCAL_BOARD_SIZE: u8 = 3;
pub const PLAYER_SYMBOLS: [PlayerSymbol; 2] = [PlayerSymbol::Cross, PlayerSymbol::Circle];

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PlayerSymbol {
  Cross = 0,
  Circle = 1,
}

impl PlayerSymbol {
  pub fn to_idx(self) -> usize {
    self as usize
  }
  pub fn from_idx(idx: usize) -> Self {
    PLAYER_SYMBOLS[idx]
  }
  pub fn other(self) -> Self {
    match self {
      Self::Cross => Self::Circle,
      Self::Circle => Self::Cross,
    }
  }
  pub fn switch(&mut self) {
    *self = self.other();
  }
}

impl Distribution<PlayerSymbol> for rand::distributions::Standard {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PlayerSymbol {
    PlayerSymbol::from_idx(rng.gen_range(0..2))
  }
}
