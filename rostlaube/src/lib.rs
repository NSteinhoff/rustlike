// use tcod::console::Root;
use tcod::console::{Root, Offscreen};
use tcod::console::{FontLayout, FontType};

// Re-export libtcod modules
pub use tcod::system;
pub use tcod::colors;
pub use tcod::console;
pub use tcod::input;
pub use tcod::map;

// Internal
pub mod rng;
pub mod ui;
pub mod geometry;

use ui::{Draw, Window};
use geometry::Location;

/// The height of the bottom panel
const PANEL_HEIGHT: i32 = 10;
/// The width of the sidebar
const SIDEBAR_PCT: i32 = 30;

/// Draw an object on the view
pub fn draw(item: &impl Draw, layer: &mut Offscreen, loc: &Location) {
    item.draw(layer, loc)
}


pub struct Engine {
    running: bool,
    root: Root,
    pub view: Window,
    pub ui: Window,
    pub messages: Window,
    pub sidebar: Window,
}

impl Engine {
    pub fn new(screen_width: i32, screen_height: i32, limit_fps: i32) -> Self {
        system::set_fps(limit_fps);
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
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
