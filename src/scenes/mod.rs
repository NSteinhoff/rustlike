use crate::colors;
use crate::game;
use crate::{BackgroundFlag, Console, Offscreen, TextAlignment};
use crate::{Direction, Game, PLAYER};
use crate::{Event, Key, KeyCode, State, Transition};

mod settings;
mod world;

pub use settings::GameSettings;

pub fn main_menu() -> settings::Screen {
    settings::Screen::MainMenu {
        player_name: Default::default(),
    }
}

pub fn game_world() -> world::Screen {
    world::Screen::GameWorld
}
