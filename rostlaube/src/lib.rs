// use tcod::console::Root;
use tcod::console::{Console, Root, Offscreen};
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

use geometry::Location;

/// Draw an object on the view
pub fn draw(item: &impl Draw, layer: &mut Offscreen, loc: &Location) {
    item.draw(layer, loc)
}


pub struct Engine {
    running: bool,
    root: Root,
}


pub trait Render {
    type State;

    fn render(&mut self, root: &mut Offscreen, state: &Self::State);
}


pub trait Interpret {
    type State;
    type Action;

    fn interpret(&self, event: &Event, state: &Self::State) -> Option<Self::Action>;
}


pub trait Draw {
    fn draw(&self, layer: &mut Offscreen, loc: &Location);
}


pub struct Window {
    pub con: Offscreen,
    pub pos: (i32, i32),
}


pub enum Layout {
    Fullscreen,
}

pub enum Event {
    KeyEvent(input::Key),
    Nothing,
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

        Engine {
            running: true,
            root: root,
        }
    }

    pub fn window(&self, layout: Layout) -> Window {
        use Layout::*;
        match layout {
            Fullscreen => Window {
                con: Offscreen::new(self.root.width(), self.root.height()),
                pos: (0, 0),
            }
        }
    }

    pub fn running(&self) -> bool {
        !self.root.window_closed() && self.running
    }

    pub fn toggle_fullscreen(&mut self) {
        let fullscreen = self.root.is_fullscreen();
        self.root.set_fullscreen(!fullscreen);
    }

    pub fn exit(&mut self) {
        // Toggle off fullscreen to avoid messing up the resolution
        self.root.set_fullscreen(false);
        self.running = false;
    }

    pub fn render<S, L>(&mut self, state: &S, layers: &mut [L])
        where L: Render<State=S> {
        self.root.set_default_background(colors::BLACK);

        let mut con = Offscreen::new(self.root.width(), self.root.height());

        for layer in layers {
            layer.render(&mut con, state);
        }

        console::blit(
            &con,
            (0, 0),
            (con.width(), con.height()),
            &mut self.root,
            (0, 0),
            1.0,
            1.0,
        );

        self.root.flush();
    }

    pub fn next_event(&mut self) -> Event {
        use Event::*;
        let key = self.root.wait_for_keypress(true);
        KeyEvent(key)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn do_something<T, S, A>(t: &mut T, s: S) -> Option<A>
        where T: std::fmt::Debug,
              T: Render<State=S>,
              T: Interpret<State=S, Action=A> {
        println!("{:?}", t);
        let mut c = Offscreen::new(1, 1);
        let e = Event::Nothing;

        t.render(&mut c, &s);
        t.interpret(&e, &s)
    }

    impl Render for bool {
        type State = i32;
        fn render(&mut self, _root: &mut Offscreen, _state: &Self::State) {}
    }

    impl Interpret for bool {
        type State = i32;
        type Action = bool;

        fn interpret(&self, _event: &Event, _state: &Self::State) -> Option<Self::Action> {
            Some(true)
        }
    }

    #[test]
    fn test_doing() {
        let mut t = true;
        let s = 10;
        let res = do_something(&mut t, s);
        assert!(res.unwrap_or(false));
    }
}
