use crate::{generic::line::LineState, PlayerSymbol, BOARD_AREA};
use std::str::FromStr;

use super::{line::Line, pos::Pos};

/// `TrivialBoard` is the bottom of the board hierarchy.
/// It is the base case of the inductive type `GenericBoard`.
pub type TrivialBoard = GenericBoard<TrivialTile>;

/// Inductive type generating the board hierarchy.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GenericBoard<TileType> {
  tiles: [TileType; 9],
  board_state: TileBoardState,
  line_states: [LineState; 8],
}

/// public methods
#[allow(private_bounds)]
impl<TileType: TileTrait> GenericBoard<TileType> {
  pub fn board_state(&self) -> TileBoardState {
    self.board_state
  }
  fn board_state_mut(&mut self) -> &mut TileBoardState {
    &mut self.board_state
  }

  fn line_state(&self, line: Line) -> LineState {
    self.line_states[line.idx()]
  }
  fn line_state_mut(&mut self, line: Line) -> &mut LineState {
    &mut self.line_states[line.idx()]
  }

  pub fn tile(&self, pos: impl Into<Pos>) -> &TileType {
    &self.tiles[pos.into().linear_idx()]
  }
  fn tile_mut(&mut self, pos: impl Into<Pos>) -> &mut TileType {
    &mut self.tiles[pos.into().linear_idx()]
  }

  /// Returns the trivial tile at the given position, by recursively walking the board hierarchy.
  fn trivial_tile(&self, pos_iter: impl IntoIterator<Item = Pos>) -> TrivialTile {
    let mut pos_iter = pos_iter.into_iter();
    let local_pos = pos_iter.next().expect("ran out of positions");
    TileTrait::trivial_tile(self.tile(local_pos), pos_iter)
  }

  pub fn could_place_symbol(&self, pos_iter: impl IntoIterator<Item = Pos>) -> bool {
    let mut pos_iter = pos_iter.into_iter();
    let local_pos = pos_iter.next().expect("ran out of positions");

    self.board_state().is_placeable()
      && TileTrait::could_place_symbol(self.tile(local_pos), pos_iter)
  }

  /// Tries to place a symbol on the given trivial tile, by recursively walking the board hierarchy and
  /// updating the state of the `TrivialTile` and the hierarchy of super states.
  pub fn try_place_symbol(
    &mut self,
    pos_iter: impl IntoIterator<Item = Pos>,
    symbol: PlayerSymbol,
  ) -> Result<(), PlaceSymbolError> {
    let mut pos_iter = pos_iter.into_iter();
    let local_pos = pos_iter.next().expect("ran out of positions");

    if self.board_state().is_placeable() {
      TileTrait::try_place_symbol(self.tile_mut(local_pos), pos_iter, symbol)?;
      self.update_super_states(local_pos);
      Ok(())
    } else {
      Err(PlaceSymbolError::BoardNotPlaceable)
    }
  }

  /// Updates the local super states (line and board states), after a tile at the given pos has changed.
  pub fn update_super_states(&mut self, local_pos: Pos) {
    for line in Line::all_through_point(local_pos) {
      let line_state = line
        .iter()
        .map(|pos| LineState::from(self.tile(pos).tile_state()))
        .reduce(|a, b| a.combine(b))
        .unwrap();

      *self.line_state_mut(line) = line_state;
      if let Some(p) = line_state.winner() {
        *self.board_state_mut() = TileBoardState::Won(p);
      }
    }
    if Line::all().all(|line| self.line_state(line).is_drawn()) {
      *self.board_state_mut() = TileBoardState::Drawn;
    }
    if Line::all().all(|line| self.line_state(line).is_fully_drawn()) {
      *self.board_state_mut() = TileBoardState::FullyDrawn;
    }
  }
}

/// `TrivialTile` is the bottom of the tile hierarchy.
#[derive(Debug, Clone, Copy, Default)]
pub enum TrivialTile {
  #[default]
  Free,
  Won(PlayerSymbol),
}

impl TrivialTile {
  pub fn is_free(self) -> bool {
    matches!(self, Self::Free)
  }
  pub fn is_won(self) -> bool {
    matches!(self, Self::Won(_))
  }

  pub fn as_char(self) -> char {
    match self {
      Self::Free => '_',
      Self::Won(p) => p.as_char(),
    }
  }
  pub fn from_char(c: char) -> Option<Self> {
    match c {
      '_' => Some(Self::Free),
      _ => PlayerSymbol::from_char(c).map(Self::Won),
    }
  }
}

/// Trait to allow recursion on inductive tile hierarchy.
trait TileTrait {
  fn tile_state(&self) -> TileBoardState;
  fn trivial_tile(&self, pos_iter: impl Iterator<Item = Pos>) -> TrivialTile;

  fn could_place_symbol(&self, pos_iter: impl Iterator<Item = Pos>) -> bool;
  fn try_place_symbol(
    &mut self,
    pos_iter: impl Iterator<Item = Pos>,
    symbol: PlayerSymbol,
  ) -> Result<(), PlaceSymbolError>;
}

/// Base case of the inductive tile hierarchy.
impl TileTrait for TrivialTile {
  fn tile_state(&self) -> TileBoardState {
    (*self).into()
  }

  fn trivial_tile(&self, mut pos_iter: impl Iterator<Item = Pos>) -> TrivialTile {
    assert!(pos_iter.next().is_none());
    *self
  }

  fn could_place_symbol(&self, mut pos_iter: impl Iterator<Item = Pos>) -> bool {
    assert!(pos_iter.next().is_none());
    self.is_free()
  }
  fn try_place_symbol(
    &mut self,
    mut pos_iter: impl Iterator<Item = Pos>,
    symbol: PlayerSymbol,
  ) -> Result<(), PlaceSymbolError> {
    assert!(pos_iter.next().is_none());
    if self.is_free() {
      *self = TrivialTile::Won(symbol);
      Ok(())
    } else {
      Err(PlaceSymbolError::TrivialTileNotFree)
    }
  }
}

/// Induction step of the inductive tile hierarchy.
impl<TileType: TileTrait> TileTrait for GenericBoard<TileType> {
  fn tile_state(&self) -> TileBoardState {
    self.board_state()
  }
  fn trivial_tile(&self, pos_iter: impl Iterator<Item = Pos>) -> TrivialTile {
    GenericBoard::trivial_tile(self, pos_iter)
  }

  fn could_place_symbol(&self, pos_iter: impl Iterator<Item = Pos>) -> bool {
    GenericBoard::could_place_symbol(self, pos_iter)
  }
  fn try_place_symbol(
    &mut self,
    pos_iter: impl Iterator<Item = Pos>,
    symbol: PlayerSymbol,
  ) -> Result<(), PlaceSymbolError> {
    GenericBoard::try_place_symbol(self, pos_iter, symbol)
  }
}

/// A `TileBoardState` is a state inside the board hierarchy.
/// It can be seen as both a tile state and a board state,
/// depending on what level of the hierarchy you are considering.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TileBoardState {
  #[default]
  Free,
  Won(PlayerSymbol),
  Drawn,
  FullyDrawn,
}

impl TileBoardState {
  pub fn is_free(self) -> bool {
    matches!(self, Self::Free)
  }
  pub fn is_won(self) -> bool {
    matches!(self, Self::Won(_))
  }
  pub fn is_drawn(self) -> bool {
    matches!(self, Self::Drawn | Self::FullyDrawn)
  }
  pub fn is_fully_drawn(self) -> bool {
    matches!(self, Self::FullyDrawn)
  }

  pub fn is_decided(self) -> bool {
    self.is_won() || self.is_drawn()
  }
  pub fn is_placeable(self) -> bool {
    !self.is_won() && !self.is_fully_drawn()
  }
}

impl From<TrivialTile> for TileBoardState {
  fn from(tile: TrivialTile) -> Self {
    match tile {
      TrivialTile::Free => Self::Free,
      TrivialTile::Won(player) => Self::Won(player),
    }
  }
}

#[derive(Debug)]
pub enum PlaceSymbolError {
  BoardNotPlaceable,
  TrivialTileNotFree,
}

#[derive(Debug)]
pub enum TrivialBoardParseError {
  InvalidChar(char),
  BadSymbolCount,
}

impl FromStr for TrivialBoard {
  type Err = TrivialBoardParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut board = TrivialBoard::default();

    let tiles: Result<Vec<_>, _> = s
      .chars()
      .filter(|c| !c.is_whitespace())
      .map(|c| TrivialTile::from_char(c).ok_or(TrivialBoardParseError::InvalidChar(c)))
      .collect();
    let tiles: [TrivialTile; BOARD_AREA as usize] = tiles?
      .try_into()
      .map_err(|_| TrivialBoardParseError::BadSymbolCount)?;

    for (i, tile) in tiles.into_iter().enumerate() {
      if let TrivialTile::Won(p) = tile {
        let pos = Pos::from_linear_idx(i);
        // TODO: maybe handle better
        board.try_place_symbol(pos.iter(), p).unwrap();
      }
    }

    Ok(board)
  }
}

#[cfg(test)]
mod test {
  use super::{TileBoardState, TrivialBoard};

  #[test]
  fn check_draw_detection() {
    let board = r#"
      XOO
      OXX
      _XO
      "#
    .parse::<TrivialBoard>()
    .unwrap();
    assert_eq!(board.board_state, TileBoardState::Drawn);
  }
}
