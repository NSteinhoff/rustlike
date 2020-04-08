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
    let mut engine = rostlaube::Engine::new(SCREEN_WIDTH, SCREEN_HEIGHT, LIMIT_FPS);

    let settings = engine.run(None, SettingsScreen::MainMenu);

    let game = settings.and_then(|settings| match settings {
        Settings::NewGame { player_name } => Some(Game::new(
            &player_name,
            Dimension(MAP_WIDTH, MAP_HEIGHT),
            Dimension(ROOM_MIN_SIZE, ROOM_MAX_SIZE),
            MAX_ROOMS,
            MAX_ROOM_MONSTERS,
            MAX_ROOM_ITEMS,
        )),
        Settings::LoadGame { path } => {
            println!("Load game from: {:?}", path);
            None
        }
    });

    engine.run_if(game, GameScreen::GameWorld).map(|game| {
        println!("Final game state:");
        println!("{:?}", game);
        game
    });

    engine.exit();
}

#[derive(Debug)]
pub enum Settings {
    NewGame { player_name: String },
    LoadGame { path: String },
}

#[derive(Debug)]
pub enum SettingsScreen {
    MainMenu,
}

impl State for SettingsScreen {
    type World = Option<Settings>;
    type Action = String;

    fn render(&self, con: &mut Offscreen, _settings: &Self::World) {
        use SettingsScreen::*;

        match self {
            MainMenu => {
                con.set_default_background(colors::BLACK);
                con.set_default_foreground(colors::WHITE);

                let (w, h) = (con.width(), con.height());

                con.print_rect_ex(
                    w / 2,
                    h / 4,
                    w - 2,
                    h - 2,
                    BackgroundFlag::Set,
                    TextAlignment::Center,
                    format!(
                        "{}\n\n{}\n\n\n\n\n{}",
                        "* Rustlike *",
                        "A short adventure in game development.",
                        "Press Enter to start a game. ESC to exit.",
                    ),
                );
            } //game.render_main_menu(con),
        }
    }

    fn interpret(
        &self,
        event: &Event,
        _settings: &Self::World,
    ) -> (Option<Self::Action>, Transition<Self>) {
        use Event::*;
        use KeyCode::{Enter, Escape};
        use SettingsScreen::*;
        use Transition::*;

        match self {
            MainMenu => match event {
                KeyEvent(Key { code: Escape, .. }) => (None, Exit),
                KeyEvent(Key { code: Enter, .. }) => (Some(String::from("Rodney")), Exit),
                _ => (None, Continue),
            },
        }
    }

    fn update(&self, action: Self::Action, settings: &mut Self::World) {
        use SettingsScreen::*;

        match self {
            MainMenu => {
                settings.replace(Settings::NewGame {
                    player_name: action,
                });
            }
        }
    }
}

#[derive(Debug)]
pub enum GameScreen {
    GameWorld,
    Console,
    Inventory,
    Character,
}

impl State for GameScreen {
    type World = Game;
    type Action = Action;

    fn render(&self, con: &mut Offscreen, game: &Self::World) {
        use GameScreen::*;

        match self {
            GameWorld => {
                game.render_game_world(con);
                game.render_messages(con);
            }
            Inventory => println!("Show inventory"),
            Character => println!("Show character"),
            Console => println!("Show console"),
        };
    }

    fn interpret(
        &self,
        event: &Event,
        game: &Self::World,
    ) -> (Option<Self::Action>, Transition<Self>) {
        use Event::*;
        use GameScreen::*;
        use KeyCode::Escape;
        use Transition::*;

        match self {
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

    fn update(&self, action: Self::Action, game: &mut Self::World) {
        use GameScreen::*;

        match self {
            GameWorld => game.update(action),
            Inventory => {}
            Character => {}
            Console => {}
        }
    }
}
