// use tcod::console::Root;
use tcod::console::{Console, Offscreen, Root};
use tcod::console::{FontLayout, FontType};

// Re-export libtcod modules
pub use tcod::colors;
pub use tcod::console;
pub use tcod::input;
pub use tcod::map;
pub use tcod::system;

// Internal
pub mod geometry;
pub mod rng;
pub mod ui;

use geometry::Location;

pub struct Engine {
    running: bool,
    root: Root,
}

pub trait Scene: std::marker::Sized {
    type State;
    type Action;

    fn render(&self, root: &mut Offscreen, state: &Self::State);

    fn interpret(
        &self,
        event: &Event,
        state: &Self::State,
    ) -> (Option<Self::Action>, Transition<Self>);

    fn update(&self, action: Self::Action, state: &mut Self::State);
}

#[derive(Debug)]
pub enum Transition<T: Scene> {
    Exit,
    Continue,
    Next(T),
    Replace(T),
}

#[derive(Debug)]
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

    pub fn run<L, S, A>(&mut self, mut state: S, start: L) -> S
    where
        A: std::fmt::Debug,
        L: std::fmt::Debug,
        L: Scene<State = S, Action = A>,
    {
        let mut scenes = vec![start];
        while self.running() {
            println!("ENGINE: scenes: {:?}", scenes);

            scenes
                .last()
                .map(|scene| {
                    println!("ENGINE: scene = {:?}", scene);
                    self.render(scene, &state);
                    scene
                })
                .and_then(|scene| {
                    let event = self.next_event();
                    println!("ENGINE: event = {:?}", event);
                    event.map(|e| (scene, e))
                })
                .map(|(scene, event)| {
                    let (action, transition) = scene.interpret(&event, &mut state);
                    println!("ENGINE: action = {:?}", action);
                    println!("ENGINE: transition = {:?}", transition);
                    action.map(|action| scene.update(action, &mut state));
                    transition
                })
                .map(|transition| match transition {
                    Transition::Continue => {}
                    Transition::Exit => {
                        scenes.pop();
                    }
                    Transition::Next(s) => scenes.push(s),
                    Transition::Replace(s) => {
                        scenes.pop();
                        scenes.push(s);
                    }
                });

            if scenes.is_empty() {
                break;
                // self.exit();
            }
        }

        state
    }

    pub fn run_if<L, S, A>(&mut self, state: Option<S>, start: L) -> Option<S>
    where
        A: std::fmt::Debug,
        L: std::fmt::Debug,
        L: Scene<State = S, Action = A>,
    {
        state.map(|s| self.run(s, start))
    }

    pub fn exit(&mut self) {
        // Toggle off fullscreen to avoid messing up the resolution
        self.root.set_fullscreen(false);
        self.running = false;
    }
}

impl Engine {
    fn render<L, S>(&mut self, layer: &L, state: &S)
    where
        L: Scene<State = S>,
    {
        self.root.set_default_background(colors::BLACK);

        let mut con = Offscreen::new(self.root.width(), self.root.height());

        layer.render(&mut con, state);

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

    fn running(&self) -> bool {
        !self.root.window_closed() && self.running
    }

    fn toggle_fullscreen(&mut self) {
        let fullscreen = self.root.is_fullscreen();
        self.root.set_fullscreen(!fullscreen);
    }

    fn next_event(&mut self) -> Option<Event> {
        use input::{Key, KeyCode};
        use Event::*;

        let key = self.root.wait_for_keypress(true);

        match key {
            Key {
                code: KeyCode::Enter,
                alt: true,
                ..
            } => {
                println!("ENGINE: Toggle Fullscreen");
                self.toggle_fullscreen();
                None
            }
            Key {
                code: KeyCode::Char,
                left_ctrl: true,
                printable: 'c',
                ..
            } => {
                println!("ENGINE: CTRL-C received -> Exit!");
                self.exit();
                None
            }
            _ => Some(KeyEvent(key)),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(1, 1);
    }
}
