pub use rostlaube::colors::{self, Color};

use rostlaube::console;
use rostlaube::input;
use rostlaube::geometry;
use rostlaube::console::{
    BackgroundFlag, Console, FontLayout, FontType, Offscreen, Root, TextAlignment,
};
pub use rostlaube::map::{FovAlgorithm, Map as FovMap};
use rostlaube::{Draw, Window}; // Re-exports only
use rostlaube::ui::Bar; // Re-exports only

use crate::game::{self, Action, Game, Messages, Object, Tile};
use crate::{Dimension, Location, PLAYER};

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

/// The height of the bottom panel
const PANEL_HEIGHT: i32 = 10;
/// The width of the sidebar
const SIDEBAR_PCT: i32 = 30;

pub struct Engine {
    running: bool,
    root: Root,
    view: Window,
    ui: Window,
    messages: Window,
    sidebar: Window,
}

impl Engine {
    pub fn new(screen_width: i32, screen_height: i32, limit_fps: i32) -> Self {
        rostlaube::system::set_fps(limit_fps);
        let mut root = Root::initializer()
            .font("src/consolas12x12.png", FontLayout::Tcod)
            .font_type(FontType::Greyscale)
            .size(screen_width, screen_height)
            .title("Rusty Roguelike")
            .init();
        root.set_fullscreen(false);

        let sidebar_width = (screen_width as f32 * (SIDEBAR_PCT as f32 / 100.0)) as i32;
        println!("{}", sidebar_width);
        let sidebar_height = screen_height;

        let sidebar_x = screen_width - sidebar_width;
        let sidebar_y = 0;

        let panel_x = sidebar_x + 2;
        let panel_y = sidebar_y + 2;
        let panel_width = sidebar_width - 2 - 2;
        let panel_height = PANEL_HEIGHT;

        let msg_x = panel_x;
        let msg_y = panel_y + panel_height + 2;
        let msg_width = sidebar_width - 2 - 2;
        let msg_height = sidebar_height - 2 - panel_height - 2 - 1;

        let view_width = screen_width - sidebar_width - 2;
        let view_height = screen_height - panel_height - 2;

        Engine {
            running: true,
            root: root,
            view: Window {
                con: Offscreen::new(view_width, view_height),
                pos: (0, 0),
            },
            ui: Window {
                con: Offscreen::new(panel_width, panel_height),
                pos: (panel_x, panel_y),
            },
            messages: Window {
                con: Offscreen::new(msg_width, msg_height),
                pos: (msg_x, msg_y),
            },
            sidebar: Window {
                con: Offscreen::new(sidebar_width, sidebar_height),
                pos: (sidebar_x, sidebar_y),
            },
        }
    }

    pub fn running(&self) -> bool {
        !self.root.window_closed() && self.running
    }

    pub fn exit(&mut self) {
        // Toggle off fullscreen to avoid messing up the resolution
        self.root.set_fullscreen(false);
        self.running = false;
    }

    pub fn next_command(&mut self) -> Command {
        let key = self.root.wait_for_keypress(true);
        get_key_command(key)
    }

    pub fn toggle_fullscreen(&mut self) {
        let fullscreen = self.root.is_fullscreen();
        self.root.set_fullscreen(!fullscreen);
    }

    pub fn render(&mut self, game: &Game) {
        self.root.set_default_background(colors::BLACK);

        self.render_view(game);
        self.render_sidebar(game);
        self.render_ui(game);
        self.render_messages(game);

        self.root.flush();
    }

    fn render_view(&mut self, game: &Game) {
        let focus = &game.objects[PLAYER].loc;

        let source = &game.map_dimensions;
        let target = &Dimension(self.view.con.width(), self.view.con.height());

        self.view.con.clear();

        let Dimension(map_width, map_height) = game.map_dimensions;
        for y_map in 0..map_height {
            for x_map in 0..map_width {
                let loc = &Location(x_map, y_map);
                let view_loc = geometry::translate(source, target, loc, focus);
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
                    self.view
                        .con
                        .set_char_background(x, y, color, BackgroundFlag::Set);
                    if let Some(c) = char {
                        self.view.con.set_default_foreground(colors::LIGHT_GREY);
                        self.view.con.put_char(x, y, *c, BackgroundFlag::None);
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
            if let Some(loc) = geometry::translate(source, target, &object.loc, focus) {
                rostlaube::draw(object, &mut self.view.con, &loc);
            }
        }

        console::blit(
            &self.view.con,
            (0, 0),
            (self.view.con.width(), self.view.con.height()),
            &mut self.root,
            self.view.pos,
            1.0,
            1.0,
        );
    }

    fn render_ui(&mut self, game: &Game) {
        let player = &game.objects[PLAYER];
        self.ui.con.set_default_background(colors::BLACK);
        self.ui.con.clear();

        if let Some(fighter) = player.fighter {
            let health_bar = Bar {
                x: 0,
                y: 0,
                color: colors::GREEN,
                background: colors::RED,
                current: fighter.health,
                maximum: fighter.max_health,
                width: self.ui.con.width(),
                name: String::from("HP"),
            };
            rostlaube::draw(&health_bar, &mut self.ui.con, &Location(0, 0));
        }

        self.ui.con.set_default_background(colors::BLACK);
        self.ui.con.set_default_foreground(colors::WHITE);
        let y = 2;
        let opponents = game::fighters_by_distance(PLAYER, &game.objects, game::TORCH_RADIUS);
        for (i, &id) in opponents
            .iter()
            .rev()
            .enumerate()
            .take(self.ui.con.height() as usize - y as usize - 1)
        // Only as many as there is space for
        {
            let o = &game.objects[id];
            if game.visible(&o.loc) {
                self.ui
                    .con
                    .put_char_ex(1, i as i32 + 1 + 1, o.char, o.color, colors::BLACK);
                self.ui.con.print_ex(
                    2,
                    i as i32 + y,
                    BackgroundFlag::None,
                    TextAlignment::Left,
                    format!(" {}", o.name),
                )
            }
        }

        console::blit(
            &self.ui.con,
            (0, 0),
            // blit the UI panel onto the screen just below the view
            (self.ui.con.width(), self.ui.con.height()),
            &mut self.root,
            self.ui.pos,
            1.0,
            1.0,
        );
    }

    fn render_sidebar(&mut self, _game: &Game) {
        self.sidebar.con.set_default_background(colors::BLACK);
        self.sidebar.con.clear();

        console::blit(
            &self.sidebar.con,
            (0, 0),
            (self.sidebar.con.width(), self.sidebar.con.height()),
            &mut self.root,
            self.sidebar.pos,
            1.0,
            1.0,
        );
    }

    fn render_messages(&mut self, game: &Game) {
        let messages = &game.messages;
        self.messages.con.set_default_background(colors::BLACK);
        self.messages.con.clear();

        rostlaube::draw(messages, &mut self.messages.con, &Location(0, 0));

        console::blit(
            &self.messages.con,
            (0, 0),
            // blit the messages onto the screen just below the view
            (self.messages.con.width(), self.messages.con.height()),
            &mut self.root,
            self.messages.pos,
            1.0,
            1.0,
        );
    }

    pub fn menu(&mut self, header: &str, options: &[&str], width: i32) -> Option<usize> {
        assert!(options.len() <= 26, "Cannot have more than 26 options");
        let header_height = self
            .root
            .get_height_rect(0, 0, width, self.root.height(), header);
        let height = header_height + options.len() as i32;
        let mut window = Offscreen::new(width, height);

        window.set_default_foreground(colors::WHITE);
        window.print_rect_ex(
            0,
            0,
            width,
            height,
            BackgroundFlag::None,
            TextAlignment::Left,
            header,
        );

        for (index, option) in options.iter().enumerate() {
            let letter = (b'a' + index as u8) as char;
            let text = format!("{} {}", letter, option);
            window.print_ex(
                0,
                header_height + index as i32,
                BackgroundFlag::None,
                TextAlignment::Left,
                text,
            );
        }

        let x = self.view.con.width() / 2 - width / 2;
        let y = self.view.con.height() / 2 - height / 2;

        console::blit(
            &window,
            (0, 0),
            (width, height),
            &mut self.root,
            (x, y),
            1.0,
            0.7,
        );
        self.root.flush();

        let key = self.root.wait_for_keypress(true);

        if key.printable.is_alphabetic() {
            let index = key.printable.to_ascii_lowercase() as usize - 'a' as usize;
            if index < options.len() {
                Some(index)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn render_main_menu(&mut self) {
        let (w, h) = (self.root.width(), self.root.height());
        let mut con = Offscreen::new(w, h);
        con.set_default_background(colors::BLACK);
        con.set_default_foreground(colors::WHITE);

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
                "Press any key to continue. ESC to exit.",
            ),
        );

        console::blit(&con, (0, 0), (w, h), &mut self.root, (0, 0), 1.0, 1.0);
        self.root.flush();
    }
}

pub fn get_key_command(key: input::Key) -> Command {
    use rostlaube::input::{Key, KeyCode::Char, KeyCode::Enter, KeyCode::Escape};
    use Command::*;
    match key {
        Key { code: Escape, .. } => Exit,
        Key {
            code: Enter,
            alt: true,
            ..
        } => ToggleFullScreen,
        Key {
            code: Char,
            printable: 'k',
            ..
        } => Up,
        Key {
            code: Char,
            printable: 'j',
            ..
        } => Down,
        Key {
            code: Char,
            printable: 'h',
            ..
        } => Left,
        Key {
            code: Char,
            printable: 'l',
            ..
        } => Right,
        Key {
            code: Char,
            printable: 'y',
            ..
        } => UpLeft,
        Key {
            code: Char,
            printable: 'u',
            ..
        } => UpRight,
        Key {
            code: Char,
            printable: 'b',
            ..
        } => DownLeft,
        Key {
            code: Char,
            printable: 'n',
            ..
        } => DownRight,
        Key {
            code: Char,
            printable: '.',
            ..
        } => Skip,
        Key {
            code: Char,
            printable: 'g',
            ..
        } => Grab,
        Key {
            code: Char,
            printable: 'i',
            ..
        } => OpenInventory,
        _ => Nothing,
    }
}

impl Draw for Object {
    /// Draw an object on the view
    fn draw(&self, layer: &mut Offscreen, loc: &Location) {
        let Location(x, y) = *loc;
        layer.set_default_foreground(self.color);
        layer.put_char(x, y, self.char, BackgroundFlag::None);
    }
}

impl Draw for Messages {
    fn draw(&self, layer: &mut Offscreen, loc: &Location) {
        let Location(x, y) = loc;
        // The width of a printed line is constrained by the width of the
        // console
        let width = layer.width() - x;

        // The maximum number of lines that we can print is equal to the height
        // of console
        let mut lines_remain = layer.height() - y;

        // We iterate through the messages in reverse in order to start with the
        // latest message
        for &(ref msg, color) in self.iter().rev() {
            // Check how many lines this message will use
            let lines = layer.get_height_rect(0, 0, width, 0, msg);
            lines_remain -= lines;
            if lines_remain < 0 {
                // The message does not fit, we have to stop here
                break;
            }
            // The vertical position is the same as the remaining lines.
            // If, for example, the message will only just fit (lines_remain == 0),
            // then it will be printed at the top of the console.
            let y = lines_remain;

            layer.set_default_foreground(color);
            layer.print_rect(0, y, width, 0, msg);
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Exit,
    Skip,
    Nothing,
    ToggleFullScreen,
    OpenInventory,

    Left,
    Right,
    Up,
    Down,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,

    Grab,
}

/// Scene transitions
///
/// Exit: Exit the current scene
/// Continue: Remain in the current scene
/// Next: Move to the next scene
pub enum Transition {
    Exit,
    Continue,
    Next(Box<dyn Scene>),
}

pub trait Scene {
    /// Update the game state
    fn update(&self, game: &mut Game, action: Action);

    /// Present the game state
    fn present(&self, engine: &mut Engine, game: &Game);
    /// Interpret command
    fn interpret(
        &self,
        engine: &mut Engine,
        game: &mut Game,
        cmd: Command,
    ) -> (Option<Action>, Transition);
}
