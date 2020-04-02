#![allow(dead_code)]
//! Architecture
//! ============
//!
//! Game Loop
//! ---------
//!
//!     +---> P ---> A ---> I ---> U ---+
//!     |                               |
//!     ^          GAME LOOP            v
//!     |                               |
//!     +---------| running? |----------+
//!
//! * `Present` the scene to the user
//! * `Accept` input from the user.
//! * `Interpret` the input of the user and determine what should happen
//!   and determines the next scene
//! * `Update` the game state based on the interpretation
//!
//! Scenes
//! ------
//! The game will present different scenes to the player:
//!
//! * Main menu
//! * Game world
//! * Level up screen
//! * Reward / path selection screen
//!
//! Scenes are pushed onto the `SceneStack` based on the results
//! of interpreting the player input. When a scene exits, it is popped
//! off the stack. A scene can also push another scene onto the stack,
//! which would then need to exit, before returning back to the original
//! scene.
//!
//!     |> Main Menu ! START
//!     => Main Menu -> Game World ! ATTACK
//!     => Main Menu -> Game World ! LEVEL UP
//!     => Main Menu -> Game World -> Level Up ! EXIT
//!     => Main Menu -> Game World ! MOVE
//!     => Main Menu -> Game World ! NEXT LEVEL
//!     => Main Menu -> Game World -> Choose Path ! OPEN INVENTORY
//!     => Main Menu -> Game World -> Choose Path -> Inventory ! EXIT
//!     => Main Menu -> Game World -> Choose Path -> ! EXIT
//!     => Main Menu -> Game World ! MOVE
//!     => Main Menu -> Game World ! EXIT
//!     => Main Menu ! EXIT
//!     <| OS

pub use rostlaube::colors::{self, Color};
pub use rostlaube::console::{BackgroundFlag, Console, Offscreen, TextAlignment};
pub use rostlaube::geometry::{Dimension, Direction, Location};
pub use rostlaube::input::{self, Key, KeyCode};
pub use rostlaube::map::{self, FovAlgorithm, Map as FovMap};
pub use rostlaube::rng;
pub use rostlaube::ui;
pub use rostlaube::{Event, Scene, Transition};

// Internal
pub mod ai;
pub mod dungeon;
pub mod engine;
pub mod game;

use crate::game::{Action, Game};

/// Width of the game screen in number of tiles
const SCREEN_WIDTH: i32 = 1920 / 10 / 2;
/// Height of the game screen in number of tiles
const SCREEN_HEIGHT: i32 = SCREEN_WIDTH / 16 * 9;
/// Frame rate limit
const LIMIT_FPS: i32 = 60;

/// Width of the map
const MAP_WIDTH: i32 = 80;
/// Height of the map
const MAP_HEIGHT: i32 = 43;

/// Maximum width/height of a room
const ROOM_MAX_SIZE: i32 = 10;
/// Minimux width/height of a room
const ROOM_MIN_SIZE: i32 = 6;
/// Maximum number of rooms
const MAX_ROOMS: i32 = 30;
/// Maximum number of monsters per room
const MAX_ROOM_MONSTERS: i32 = 3;
/// Maximum number of items per room
const MAX_ROOM_ITEMS: i32 = 2;

/// Index of player in vector of objects
const PLAYER: usize = 0; // The player will always be the first object

/// Main entry point
fn main() {
    let game = Game::new(
        "Rodney",
        Dimension(MAP_WIDTH, MAP_HEIGHT),
        Dimension(ROOM_MIN_SIZE, ROOM_MAX_SIZE),
        MAX_ROOMS,
        MAX_ROOM_MONSTERS,
        MAX_ROOM_ITEMS,
    );
    rostlaube::Engine::new(SCREEN_WIDTH, SCREEN_HEIGHT, LIMIT_FPS).run(game, Screen::MainMenu);
}

#[derive(Debug)]
pub enum Screen {
    MainMenu,
    GameWorld,
    Console,
    Inventory,
    Character,
}

impl Scene for Screen {
    type State = Game;
    type Action = Action;

    fn render(&self, con: &mut Offscreen, game: &Game) {
        use Screen::*;

        match self {
            GameWorld => {
                game.render_game_world(con);
                game.render_messages(con);
            }
            MainMenu => game.render_main_menu(con),
            Inventory => println!("Show inventory"),
            Character => println!("Show character"),
            Console => println!("Show console"),
        };
    }

    fn interpret(
        &self,
        event: &Event,
        game: &mut Game,
    ) -> (Option<Self::Action>, Transition<Self>) {
        use Event::*;
        use KeyCode::{Enter, Escape};
        use Screen::*;
        use Transition::*;

        match self {
            MainMenu => match event {
                KeyEvent(Key { code: Escape, .. }) => (None, Exit),
                KeyEvent(Key { code: Enter, .. }) => (None, Next(GameWorld)),
                _ => (None, Continue),
            },
            GameWorld => match event {
                KeyEvent(Key { code: Escape, .. }) => (None, Exit),
                KeyEvent(key) => game.action(key),
                _ => (None, Continue),
            },
            Inventory => (None, Exit),
            Character => (None, Exit),
            Console => (None, Exit),
        }
    }

    fn update(&self, action: Action, game: &mut Game) {
        use Screen::*;

        match self {
            MainMenu => {}
            GameWorld => game.update(action),
            Inventory => {}
            Character => {}
            Console => {}
        }
    }
}
