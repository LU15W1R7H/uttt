use common::{
  specific::{
    game::GameState,
    pos::{GlobalPos, InnerPos, OuterPos},
  },
  PlayerSymbol,
};

use eframe::egui;

pub fn player_color(player: PlayerSymbol) -> egui::Color32 {
  match player {
    PlayerSymbol::X => egui::Color32::RED,
    PlayerSymbol::O => egui::Color32::BLUE,
  }
}

pub fn lightened_color(color: egui::Color32, amount: u8) -> egui::Color32 {
  egui::Color32::from_rgb(
    color.r().saturating_add(amount),
    color.g().saturating_add(amount),
    color.b().saturating_add(amount),
  )
}

pub fn choose_random_tile(game_state: &GameState) -> GlobalPos {
  use rand::Rng;
  let mut rng = rand::thread_rng();
  let outer_pos = game_state.current_outer_pos().unwrap_or_else(|| loop {
    println!("choosing inner board");
    let outer_pos = OuterPos::new(rng.gen_range(0..3), rng.gen_range(0..3));
    if game_state
      .board()
      .tile(outer_pos)
      .board_state()
      .is_placeable()
    {
      break outer_pos;
    }
  });
  let inner_pos = loop {
    println!("choosing tile in inner board {:?}", outer_pos);
    let inner_pos = InnerPos::new(rng.gen_range(0..3), rng.gen_range(0..3));
    if game_state.board().tile(outer_pos).tile(inner_pos).is_free() {
      break inner_pos;
    }
  };
  GlobalPos::from((outer_pos, inner_pos))
}