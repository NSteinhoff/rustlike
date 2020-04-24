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
//! States
//! ------
//! The game will present different scenes to the player:
//!
//! * Main menu
//! * Game world
//! * Level up screen
//! * Reward / path selection screen
//!
//! States are pushed onto the `StateStack` based on the results
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
pub use rostlaube::{Event, State, Transition};

// Internal
pub mod ai;
pub mod dungeon;
pub mod engine;
pub mod game;
mod scenes;

use crate::game::Game;
use scenes::GameSettings;

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
pub fn run() {
    let mut engine = rostlaube::Engine::new(SCREEN_WIDTH, SCREEN_HEIGHT, LIMIT_FPS);

    engine
        .run(Default::default(), scenes::main_menu())
        .and_then(|settings| match settings {
            GameSettings::NewGame { player_name } => Some(Game::new(
                &player_name,
                Dimension(MAP_WIDTH, MAP_HEIGHT),
                Dimension(ROOM_MIN_SIZE, ROOM_MAX_SIZE),
                MAX_ROOMS,
                MAX_ROOM_MONSTERS,
                MAX_ROOM_ITEMS,
            )),
            GameSettings::LoadGame { path } => {
                println!("Load game from: {:?}", path);
                None
            }
        })
        .map(|game| engine.run(game, scenes::game_world()))
        .map(|game| {
            println!("Final game state:");
            println!("{:?}", game);
        });

    engine.exit();
}
