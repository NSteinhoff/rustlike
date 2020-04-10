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

use crate::game::Game;

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

    let settings = engine.run(None, SettingsScreen::main());

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
    MainMenu { player_name: String },
}

impl SettingsScreen {
    fn main() -> Self {
        Self::MainMenu { player_name: Default::default() }
    }
}

#[derive(Debug)]
pub enum SettingsAction {
    Cancel,
    StartGame,
    ReadChar(char, bool),
    DeleteChar,
    InvalidKey,
}

impl State for SettingsScreen {
    type World = Option<Settings>;
    type Action = SettingsAction;

    fn render(&self, con: &mut Offscreen, _settings: &Self::World) {
        use SettingsScreen::*;

        match self {
            MainMenu { player_name, .. } => {
                con.set_default_background(colors::BLACK);
                con.set_default_foreground(colors::WHITE);

                let (w, h) = (con.width(), con.height());

                let text = format!(
                    "{}\n\n{}\n\n\n\n\n{}",
                    "* Rustlike *",
                    "A short adventure in game development.",
                    "Press Enter to start a game. ESC to exit.",
                );

                con.print_rect_ex(
                    w / 2,
                    h / 4,
                    w - 2,
                    h - 2,
                    BackgroundFlag::Set,
                    TextAlignment::Center,
                    &text,
                );

                let num_lines_intro = con.get_height_rect(
                    w / 2,
                    h / 4,
                    w - 2,
                    h - 2,
                    &text,
                );

                con.print_ex(
                    w / 2,
                    h / 4 + num_lines_intro + 3,
                    BackgroundFlag::Set,
                    TextAlignment::Center,
                    format!("Enter name:\n{}", player_name),
                );
            }
        }
    }

    fn interpret(
        &self,
        event: &Event,
    ) -> Self::Action {
        use Event::*;
        use KeyCode::{Char, Spacebar, Enter, Escape, Backspace};
        use SettingsScreen::*;
        use SettingsAction::*;

        match self {
            MainMenu { .. } => match event {
                KeyEvent(Key { code: Escape, .. }) => Cancel,
                KeyEvent(Key { code: Enter, .. }) => StartGame,
                KeyEvent(Key { code: Backspace, .. }) => DeleteChar,
                KeyEvent(Key { code: Spacebar, printable, .. }) => ReadChar(*printable, false),
                KeyEvent(Key { code: Char, printable, shift, .. }) => ReadChar(*printable, *shift),
                _ => InvalidKey,
            },
        }
    }

    fn update(&mut self, action: Self::Action, settings: &mut Self::World) -> Transition<Self> {
        use SettingsScreen::*;
        use SettingsAction::*;
        use Transition::*;

        match self {
            MainMenu { player_name, .. } => match action {
                StartGame => {
                    settings.replace(Settings::NewGame { player_name: player_name.clone() });
                    Exit
                },
                DeleteChar => {
                    player_name.pop();
                    Continue
                }
                ReadChar(c, upper) => {
                    if upper {
                        for u in c.to_uppercase() {
                            player_name.push(u);
                        }
                    } else {
                        player_name.push(c);
                    }
                    Continue
                }
                Cancel => Exit,
                InvalidKey => Continue
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

#[derive(Debug)]
pub enum GameScreenAction {
    Nothing,
    Exit,
    OpenInventory,
    OpenCharacterScreen,
    GameAction(game::Action),
}

impl GameScreenAction {
    fn game_action(c: &char) -> Self {
        use game::Action::*;
        let a = match c {
            'k' => Move(PLAYER, Direction(0, -1)),
            'j' => Move(PLAYER, Direction(0, 1)),
            'h' => Move(PLAYER, Direction(-1, 0)),
            'l' => Move(PLAYER, Direction(1, 0)),
            'y' => Move(PLAYER, Direction(-1, -1)),
            'u' => Move(PLAYER, Direction(1, -1)),
            'b' => Move(PLAYER, Direction(-1, 1)),
            'n' => Move(PLAYER, Direction(1, 1)),
            _ => game::Action::Nothing,
        };
        Self::GameAction(a)
    }
}

impl State for GameScreen {
    type World = Game;
    type Action = GameScreenAction;

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
    ) -> Self::Action {
        use Event::*;
        use GameScreen::*;
        use GameScreenAction::*;
        use KeyCode::{Escape, Char};

        match self {
            GameWorld => match event {
                KeyEvent(Key { code: Escape, .. }) => Exit,
                KeyEvent(Key { code: Char, printable: 'i', .. }) => OpenInventory,
                KeyEvent(Key { code: Char, printable: 'c', .. }) => OpenCharacterScreen,
                KeyEvent(Key {code: Char, printable: c, .. }) => GameScreenAction::game_action(c),
                KeyEvent(_) | Event::Nothing => GameScreenAction::Nothing,
            },
            Inventory => Exit,
            Character => Exit,
            Console => Exit,
        }
    }

    fn update(&mut self, action: Self::Action, game: &mut Self::World) -> Transition<Self> {
        use GameScreen::*;
        use GameScreenAction::*;

        match self {
            GameWorld => match action {
                Exit => Transition::Exit,
                Nothing => Transition::Continue,
                OpenInventory => Transition::Next(Inventory),
                OpenCharacterScreen => Transition::Next(Character),
                GameAction(action) => {
                    game.update(action);
                    Transition::Continue
                }
            }
            Inventory => Transition::Exit,
            Character => Transition::Exit,
            Console => Transition::Exit,
        }
    }
}
