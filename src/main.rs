#![allow(dead_code)]
// Internal
pub mod ai;
pub mod engine;
pub mod game;
pub mod rng;
pub mod dungeon;

use crate::engine::{Engine, Command, colors};
use crate::game::{Game, Object, Messages};


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


/// Main entry point
fn main() {
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
    game.messages.add("You've stumbled into some very rusty caves. Prepare yourself.", colors::GREEN);

    // Create the game engine
    let mut engine = Engine::new(
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        LIMIT_FPS,
    );

    println!("Number of monsters: {}", game.objects.len() - 1);
    println!("--- [{}] ---", game.turn + 1);
    game.messages.add(format!("--- [{}] ---", game.turn + 1), colors::WHITE);
    let messages = game.update(false);
    game.messages.append(messages);

    let mut player_turn: game::Turn = vec![];
    while engine.running() {
        engine.render(&game);

        let (action, messages) = match engine.next_command() {
            // System
            Command::Nothing => (None, Messages::empty()),
            Command::ToggleFullScreen => {
                engine.toggle_fullscreen();
                (None, Messages::new("Fullscreen toggled", colors::WHITE))
            }
            Command::Exit => {
                engine.exit();
                println!("Game turns: {}", game.turn);
                break;
            }
            // Handle the remaining commands as player actions
            command => game.player_turn(&command, &mut engine),
        };
        game.messages.append(messages);

        // Some commands do not result in the player performing
        // an action. In that case no turns are played out.
        if let Some(action) = action {

            // First play the player turn
            player_turn.push(action);
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
                println!("{}: {:?}", game.turn + 1, (&player_turn, &ai_turns));
                game.turn(player_turn.clone(), ai_turns);
                player_turn.clear();

                // Start the turn
                println!("--- [{}] ---", game.turn + 1);
                game.messages.add(format!("--- [{}] ---", game.turn + 1), colors::WHITE);
            }
        }
    }
}
