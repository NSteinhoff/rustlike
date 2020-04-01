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

use rostlaube::{Window, Layout, Event, Render};
pub use rostlaube::rng;
pub use rostlaube::geometry::{Location, Direction, Dimension};
use rostlaube::colors::{self, Color};
use rostlaube::console::{Console, Offscreen, BackgroundFlag, TextAlignment};
use rostlaube::ui::Bar;

// Internal
pub mod ai;
pub mod dungeon;
pub mod engine;
pub mod game;

use crate::engine::{Engine, Command, Scene, Transition, get_key_command};
use crate::game::{Action, Game, Messages, Object, Tile};

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

/// Color used for unexplored areas
const COLOR_UNEXPLORED: Color = colors::BLACK;
/// Color used for dark walls
const COLOR_DARK_WALL: Color = colors::DARKEST_GREY;
/// Color used for light walls
const COLOR_LIGHT_WALL: Color = colors::DARKER_GREY;
/// Color used for dark ground
const COLOR_DARK_GROUND: Color = colors::DARKER_GREY;
/// Color used for light ground
const COLOR_LIGHT_GROUND: Color = colors::DARK_GREY;

/// Index of player in vector of objects
const PLAYER: usize = 0; // The player will always be the first object


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
    let mut engine2 = rostlaube::Engine::new(SCREEN_WIDTH, SCREEN_HEIGHT, LIMIT_FPS);

    println!("Number of monsters: {}", game.objects.len() - 1);
    println!("--- [{}] ---", game.turn + 1);
    // game.messages.add(format!("--- [{}] ---", game.turn + 1), colors::WHITE);
    let messages = game.update(false);
    game.messages.append(messages);

    let mut layers = vec![
        Screen::game_world(&engine2)
    ];

    while engine2.running() {
        scenes.last().map(|scene| {
            // scene.present(&mut engine, &game);
            engine2.render(&game, &mut layers);

            if let Event::KeyEvent(key) = engine2.next_event() {
                let command = get_key_command(key);

                let (action, transition) = scene.interpret(&mut engine, &mut game, command);

                action.map(|action| scene.update(&mut game, action));

                transition
            } else {
                Transition::Continue
            }
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
            engine2.exit();
            println!("Game turns: {}", game.turn);
        }
    }
}

enum View {
    MainMenu,
    GameWorld,
}

struct Screen {
    window: Window,
    view: View
}

impl Screen {
    fn main_menu(engine: &rostlaube::Engine) -> Self {
        Self {
            window: engine.window(Layout::Fullscreen),
            view: View::MainMenu,
        }
    }

    fn game_world(engine: &rostlaube::Engine) -> Self {
        Self {
            window: engine.window(Layout::Fullscreen),
            view: View::GameWorld,
        }
    }

    fn render_game_world(&mut self, game: &Game) {
        let focus = &game.objects[PLAYER].loc;

        let source = &game.map_dimensions;
        let target = &Dimension(self.window.con.width(), self.window.con.height());

        let Dimension(map_width, map_height) = game.map_dimensions;
        for y_map in 0..map_height {
            for x_map in 0..map_width {
                let loc = &Location(x_map, y_map);
                let view_loc = rostlaube::geometry::translate(source, target, loc, focus);
                if let Some(Location(x, y)) = view_loc {
                    let tile = &game.map[x_map as usize][y_map as usize];
                    let (color, char) = match (tile.explored, tile.visible, tile) {
                        (
                            true,
                            true,
                            Tile {
                                blocked: true,
                                char: c,
                                ..
                            },
                        ) => (COLOR_LIGHT_WALL, Some(c)),
                        (true, false, Tile { blocked: true, .. }) => (COLOR_DARK_WALL, None),
                        (
                            true,
                            true,
                            Tile {
                                blocked: false,
                                char: c,
                                ..
                            },
                        ) => (COLOR_LIGHT_GROUND, Some(c)),
                        (true, false, Tile { blocked: false, .. }) => (COLOR_DARK_GROUND, None),
                        (false, _, _) => (COLOR_UNEXPLORED, None),
                    };
                    self.window
                        .con
                        .set_char_background(x, y, color, BackgroundFlag::Set);
                    if let Some(c) = char {
                        self.window.con.set_default_foreground(colors::LIGHT_GREY);
                        self.window.con.put_char(x, y, *c, BackgroundFlag::None);
                    }
                }
            }
        }

        // Sort the object to draw such that non-blocking objects are
        // drawn first to avoid drawing them over other objects standing
        // on top of them.
        let mut to_draw: Vec<_> = game.objects.iter().filter(|o| o.visible).collect();

        to_draw.sort_by(|a, b| a.blocks.cmp(&b.blocks));
        for object in to_draw {
            if let Some(loc) = rostlaube::geometry::translate(source, target, &object.loc, focus) {
                rostlaube::draw(object, &mut self.window.con, &loc);
            }
        }
    }

    fn render_ui(&mut self, game: &Game) {
        let player = &game.objects[PLAYER];
        self.window.con.set_default_background(colors::BLACK);
        self.window.con.clear();

        if let Some(fighter) = player.fighter {
            let health_bar = Bar {
                x: 0,
                y: 0,
                color: colors::GREEN,
                background: colors::RED,
                current: fighter.health,
                maximum: fighter.max_health,
                width: self.window.con.width(),
                name: String::from("HP"),
            };
            rostlaube::draw(&health_bar, &mut self.window.con, &Location(0, 0));
        }

        self.window.con.set_default_background(colors::BLACK);
        self.window.con.set_default_foreground(colors::WHITE);
        let y = 2;
        let opponents = game::fighters_by_distance(PLAYER, &game.objects, game::TORCH_RADIUS);
        for (i, &id) in opponents
            .iter()
            .rev()
            .enumerate()
            .take(self.window.con.height() as usize - y as usize - 1)
        // Only as many as there is space for
        {
            let o = &game.objects[id];
            if game.visible(&o.loc) {
                self.window
                    .con
                    .put_char_ex(1, i as i32 + 1 + 1, o.char, o.color, colors::BLACK);
                self.window.con.print_ex(
                    2,
                    i as i32 + y,
                    BackgroundFlag::None,
                    TextAlignment::Left,
                    format!(" {}", o.name),
                )
            }
        }
    }

    fn render_sidebar(&mut self, _game: &Game) {
        self.window.con.set_default_background(colors::BLACK);
        self.window.con.clear();
    }

    fn render_messages(&mut self, game: &Game) {
        let messages = &game.messages;
        self.window.con.set_default_background(colors::BLACK);
        // self.window.con.clear();

        rostlaube::draw(messages, &mut self.window.con, &Location(0, 0));
    }

    fn render_main_menu(&mut self, _game: &Game) {
        let (w, h) = (self.window.con.width(), self.window.con.height());
        self.window.con.set_default_background(colors::BLACK);
        self.window.con.set_default_foreground(colors::WHITE);

        self.window.con.print_rect_ex(
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
                "Press any key to self.window.continue. ESC to exit.",
            ),
        );
    }
}

impl Render for Screen {
    type State = Game;

    fn render(&mut self, con: &mut Offscreen, game: &Game) {
        use View::*;
        self.window.con.clear();

        match self.view {
            GameWorld => {
                self.render_game_world(game);
                self.render_messages(game);
            }
            MainMenu => self.render_main_menu(game),
        }

        rostlaube::console::blit(
            &self.window.con,
            (0, 0),
            (self.window.con.width(), self.window.con.height()),
            con,
            self.window.pos,
            1.0,
            1.0,
        );
    }
}
