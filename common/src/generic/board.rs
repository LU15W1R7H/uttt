use crate::{generic::line::LineState, PlayerSymbol};

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

// reexporting private methods that should be public
// (needed because trait methods can't be private)
#[allow(private_bounds)]
impl<TileType: TileTrait> GenericBoard<TileType> {
  pub fn board_state(&self) -> TileBoardState {
    BoardTrait::board_state(self)
  }
  pub fn tile(&self, pos: impl Into<Pos>) -> &TileType {
    BoardTrait::tile(self, pos.into())
  }
  pub fn trivial_tile(&self, pos_iter: impl IntoIterator<Item = Pos>) -> TrivialTile {
    BoardTrait::trivial_tile(self, pos_iter.into_iter())
  }
  pub fn could_place_symbol(&self, pos: impl IntoIterator<Item = Pos>) -> bool {
    BoardTrait::could_place_symbol(self, pos.into_iter())
  }
  pub fn try_place_symbol(
    &mut self,
    pos: impl IntoIterator<Item = Pos>,
    symbol: PlayerSymbol,
  ) -> bool {
    BoardTrait::try_place_symbol(self, pos.into_iter(), symbol)
  }
}

/// Trait to allow recursion on inductive board hierarchy.
trait BoardTrait {
  type TileType: TileTrait;

  fn tile(&self, pos: Pos) -> &Self::TileType;
  fn tile_mut(&mut self, pos: Pos) -> &mut Self::TileType;

  fn board_state(&self) -> TileBoardState;
  fn board_state_mut(&mut self) -> &mut TileBoardState;

  fn line_state(&self, line: Line) -> LineState;
  fn line_state_mut(&mut self, line: Line) -> &mut LineState;

  /// Returns the trivial tile at the given position, by recursively walking the board hierarchy.
  fn trivial_tile(&self, mut pos_iter: impl Iterator<Item = Pos>) -> TrivialTile {
    let local_pos = pos_iter.next().expect("ran out of positions");
    TileTrait::trivial_tile(self.tile(local_pos), pos_iter)
  }

  fn could_place_symbol(&self, mut pos_iter: impl Iterator<Item = Pos>) -> bool {
    let local_pos = pos_iter.next().expect("ran out of positions");

    self.board_state().is_placeable()
      && TileTrait::could_place_symbol(self.tile(local_pos), pos_iter)
  }

  /// Places a symbol on the given trivial tile, by recursively walking the board hierarchy and
  /// updating the state of the `TrivialTile` and the hierarchy of super states.
  fn try_place_symbol(
    &mut self,
    mut pos_iter: impl Iterator<Item = Pos>,
    symbol: PlayerSymbol,
  ) -> bool {
    let local_pos = pos_iter.next().expect("ran out of positions");

    if self.board_state().is_placeable()
      && TileTrait::try_place_symbol(self.tile_mut(local_pos), pos_iter, symbol)
    {
      self.update_super_states(local_pos);
      true
    } else {
      false
    }
  }

  /// Updates the local super states (line and board states), after a tile at the given pos has changed.
  fn update_super_states(&mut self, local_pos: Pos) {
    for line in Line::all_through_point(local_pos) {
      let line_state = line
        .iter()
        .map(|pos| LineState::from(self.tile(pos).tile_state()))
        .reduce(|a, b| a.combinator(b))
        .unwrap();

      *self.line_state_mut(line) = line_state;
      if let LineState::Won(p) = line_state {
        *self.board_state_mut() = TileBoardState::Won(p);
      }
    }
    if Line::all().all(|line| self.line_state(line).is_drawn()) {
      *self.board_state_mut() = TileBoardState::Drawn;
    }
  }
}

/// The only implementation of `BoardTrait` is for `GenericBoard`.
impl<TileType: TileTrait> BoardTrait for GenericBoard<TileType> {
  type TileType = TileType;

  fn tile(&self, pos: Pos) -> &Self::TileType {
    &self.tiles[pos.linear_idx()]
  }
  fn tile_mut(&mut self, pos: Pos) -> &mut Self::TileType {
    &mut self.tiles[pos.linear_idx()]
  }

  fn board_state(&self) -> TileBoardState {
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
}

/// Trait to allow recursion on inductive tile hierarchy.
trait TileTrait {
  fn tile_state(&self) -> TileBoardState;
  fn trivial_tile(&self, pos_iter: impl Iterator<Item = Pos>) -> TrivialTile;

  fn could_place_symbol(&self, pos_iter: impl Iterator<Item = Pos>) -> bool;
  fn try_place_symbol(&mut self, pos_iter: impl Iterator<Item = Pos>, symbol: PlayerSymbol)
    -> bool;
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
  ) -> bool {
    assert!(pos_iter.next().is_none());
    if self.is_free() {
      *self = TrivialTile::Won(symbol);
      true
    } else {
      false
    }
  }
}

/// Induction step of the inductive tile hierarchy.
impl<BoardType: BoardTrait> TileTrait for BoardType {
  fn tile_state(&self) -> TileBoardState {
    self.board_state()
  }
  fn trivial_tile(&self, pos_iter: impl Iterator<Item = Pos>) -> TrivialTile {
    BoardTrait::trivial_tile(self, pos_iter)
  }

  fn could_place_symbol(&self, pos_iter: impl Iterator<Item = Pos>) -> bool {
    BoardTrait::could_place_symbol(self, pos_iter)
  }
  fn try_place_symbol(
    &mut self,
    pos_iter: impl Iterator<Item = Pos>,
    symbol: PlayerSymbol,
  ) -> bool {
    BoardTrait::try_place_symbol(self, pos_iter, symbol)
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
}

impl TileBoardState {
  pub fn is_free(self) -> bool {
    matches!(self, Self::Free)
  }
  pub fn is_won(self) -> bool {
    matches!(self, Self::Won(_))
  }
  pub fn is_drawn(self) -> bool {
    matches!(self, Self::Drawn)
  }
  pub fn is_decided(self) -> bool {
    self.is_won() || self.is_drawn()
  }
  pub fn is_placeable(self) -> bool {
    !self.is_won()
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
}

impl From<TrivialTile> for TileBoardState {
  fn from(tile: TrivialTile) -> Self {
    match tile {
      TrivialTile::Free => Self::Free,
      TrivialTile::Won(player) => Self::Won(player),
    }
  }
}

#[cfg(test)]
mod test {
  use super::TrivialBoard;
  use crate::{
    generic::{board::TileBoardState, pos::Pos},
    PlayerSymbol,
  };

  // XOO
  // OXX
  // _XO
  // is a draw
  #[test]
  fn check_draw_detection() {
    let mut board = TrivialBoard::default();
    let moves = vec![
      (Pos::new(0, 0), PlayerSymbol::X),
      (Pos::new(1, 0), PlayerSymbol::O),
      (Pos::new(2, 0), PlayerSymbol::O),
      (Pos::new(0, 1), PlayerSymbol::O),
      (Pos::new(1, 1), PlayerSymbol::X),
      (Pos::new(2, 1), PlayerSymbol::X),
      //(Pos::new(0, 2), PlayerSymbol::Empty),
      (Pos::new(1, 2), PlayerSymbol::X),
      (Pos::new(2, 2), PlayerSymbol::O),
    ];
    for (pos, sym) in moves {
      assert!(board.try_place_symbol(pos.iter(), sym));
    }
    assert_eq!(board.board_state, TileBoardState::Drawn);
  }
}
