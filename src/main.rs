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

// Internal
pub mod ai;
pub mod dungeon;
pub mod engine;
pub mod game;
pub mod rng;

use crate::engine::{colors, Command, Engine, Scene, Transition};
use crate::game::{Action, Game, Messages, Object};

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

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Location(pub i32, pub i32);
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Direction(pub i32, pub i32);
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Dimension(pub i32, pub i32);


struct Main {}
impl Scene for Main {
    /// Update the game state
    fn update(&self, game: &mut Game, action: Action) {
        // First play the player turn
        game.player_turn.push(action);
        game.play(&vec![action]);

        let messages = game.update(false);
        game.messages.append(messages);

        // Some actions don't consume a turn
        if action.took_turn() {
            // Calculate the reaction of the AI and play
            // the AI turn.
            let ai_turns = game.ai_turns();
            game.play(&ai_turns);

            let messages = game.update(true);
            game.messages.append(messages);

            // Record the turn
            println!("{}: {:?}", game.turn + 1, (&game.player_turn, &ai_turns));
            game.turn(game.player_turn.clone(), ai_turns);

            // Start the turn
            println!("--- [{}] ---", game.turn + 1);
            // game.messages.add(format!("--- [{}] ---", game.turn + 1), colors::WHITE);
        }
    }

    /// Present the game state
    fn present(&self, engine: &mut Engine, game: &Game) {
        engine.render(game)
    }

    /// Accept input from the player
    fn accept(&self, engine: &mut Engine) -> Command {
        engine.next_command()
    }

    /// Interpret command
    fn interpret(
        &self,
        engine: &mut Engine,
        game: &mut Game,
        command: Command,
    ) -> (Option<Action>, Transition) {
        let (action, messages, transition) = match command {
            // System
            Command::Nothing => (None, Messages::empty(), Transition::Continue),
            Command::ToggleFullScreen => {
                engine.toggle_fullscreen();
                (None, Messages::new("Fullscreen toggled", colors::WHITE), Transition::Continue)
            }
            Command::Exit => {
                (None, Messages::new("Ok, bye!", colors::WHITE), Transition::Exit)
            }
            // Handle the remaining commands as player actions
            command => {
                let (a, m) = game.player_turn(&command, engine);
                (a, m, Transition::Continue)
            }
        };
        game.messages.append(messages);
        (action, transition)
    }
}

struct MainMenu {}
impl Scene for MainMenu {
    /// Update the game state
    fn update(&self, _game: &mut Game, _action: Action) {}

    /// Present the game state
    fn present(&self, engine: &mut Engine, _game: &Game) {
        engine.render_main_menu()
    }

    /// Accept input from the player
    fn accept(&self, engine: &mut Engine) -> Command {
        engine.next_command()
    }

    /// Interpret command
    fn interpret(
        &self,
        _engine: &mut Engine,
        _game: &mut Game,
        command: Command,
    ) -> (Option<Action>, Transition) {
        match command {
            Command::Exit => (None, Transition::Exit),
            _ => (None, Transition::Next(Box::new(Main {}))),
        }
    }
}


/// Main entry point
fn main() {
    let mut scenes: Vec<Box<dyn Scene>> = vec![];
    scenes.push(Box::new(MainMenu {}));

    // Create a player and an NPC
    let player = Object::player(Location(0, 0), "Rodney");

    // Create the game
    let mut game = Game::new(
        player,
        Dimension(MAP_WIDTH, MAP_HEIGHT),
        Dimension(ROOM_MIN_SIZE, ROOM_MAX_SIZE),
        MAX_ROOMS,
        MAX_ROOM_MONSTERS,
        MAX_ROOM_ITEMS,
    );
    game.messages.add(
        "You've stumbled into some very rusty caves. Prepare yourself.",
        colors::GREEN,
    );

    // Create the game engine
    let mut engine = Engine::new(SCREEN_WIDTH, SCREEN_HEIGHT, LIMIT_FPS);

    println!("Number of monsters: {}", game.objects.len() - 1);
    println!("--- [{}] ---", game.turn + 1);
    // game.messages.add(format!("--- [{}] ---", game.turn + 1), colors::WHITE);
    let messages = game.update(false);
    game.messages.append(messages);

    while engine.running() {
        scenes.last().map(|scene| {
            scene.present(&mut engine, &game);
            let command = scene.accept(&mut engine);
            let (action, transition) = scene.interpret(&mut engine, &mut game, command);
            action.map(|action| scene.update(&mut game, action));
            transition
        })
        .map(|transition| {
            match transition {
                Transition::Continue => {},
                Transition::Exit => { scenes.pop(); },
                Transition::Next(s) => scenes.push(s),
            }
        });

        println!("Scene Stack: {}", scenes.len());
        if scenes.is_empty() {
            engine.exit();
            println!("Game turns: {}", game.turn);
        }
    }
}
