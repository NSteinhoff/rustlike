pub use rostlaube::colors::{self, Color};

use crate::ui::{self, Bar, Draw};
use rostlaube::console;
use rostlaube::console::{
    BackgroundFlag, Console, FontLayout, FontType, Offscreen, Root, TextAlignment,
};
pub use rostlaube::map::{FovAlgorithm, Map as FovMap};

use crate::game::{self, Game, Messages, Object};
use crate::{Location, PLAYER};

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

struct Window {
    pub con: Offscreen,
    pub pos: (i32, i32),
}

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
            ui::draw(&health_bar, &mut self.ui.con, &Location(0, 0));
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

        ui::draw(messages, &mut self.messages.con, &Location(0, 0));

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
