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
pub mod pda;
pub mod command_line;

use geometry::Location;
use command_line::CommandLine;

pub struct Engine {
    running: bool,
    root: Root,
}

pub trait State: std::marker::Sized + std::fmt::Debug {
    type World;
    type Action;

    fn render(&self, root: &mut Offscreen, world: &Self::World);

    fn interpret(
        &self,
        event: &Event,
    ) -> Self::Action;

    fn update(&mut self, action: Self::Action, world: &mut Self::World) -> Transition<Self>;
}

#[derive(Debug)]
pub enum Transition<S: State> {
    Exit,
    Continue,
    Next(S),
    Replace(S),
}

#[derive(Debug)]
pub enum Event {
    KeyEvent(input::Key),
    Command(String),
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

    pub fn run<S, W, A>(&mut self, mut world: W, start: S) -> W
    where
        A: std::fmt::Debug,
        S: std::fmt::Debug,
        S: State<World = W, Action = A>,
    {
        let mut scenes = vec![start];
        while self.running() {
            println!("ENGINE: scenes: {:?}", scenes);

            scenes
                .pop()
                .map(|scene| {
                    println!("ENGINE: scene = {:?}", scene);
                    self.render(&scene, &world);
                    scene
                })
                .and_then(|scene| {
                    let event = self.next_event();
                    println!("ENGINE: event = {:?}", event);
                    event.map(|e| (scene, e))
                })
                .map(|(scene, event)| {
                    let action = scene.interpret(&event);
                    println!("ENGINE: action = {:?}", action);
                    (scene, action)
                })
                .map(|(mut scene, action)| {
                    let transition = scene.update(action, &mut world);
                    println!("ENGINE: transition = {:?}", transition);
                    (scene, transition)
                })
                .map(|(scene, transition)| match transition {
                    Transition::Continue => {
                        scenes.push(scene);
                    }
                    Transition::Exit => {},
                    Transition::Next(s) => {
                        scenes.push(scene);
                        scenes.push(s);
                    },
                    Transition::Replace(s) => {
                        scenes.push(s);
                    },
                });

            if scenes.is_empty() {
                break;
                // self.exit();
            }
        }

        world
    }

    pub fn run_if<S, W, A>(&mut self, world: Option<W>, start: S) -> Option<W>
    where
        A: std::fmt::Debug,
        S: State<World = W, Action = A> + std::fmt::Debug,
    {
        world.map(|s| self.run(s, start))
    }

    pub fn exit(&mut self) {
        // Toggle off fullscreen to avoid messing up the resolution
        self.root.set_fullscreen(false);
        self.running = false;
    }
}

impl Engine {
    fn render<S, W>(&mut self, layer: &S, world: &W)
    where
        S: State<World = W>,
    {
        self.root.set_default_background(colors::BLACK);

        let mut con = Offscreen::new(self.root.width(), self.root.height());

        layer.render(&mut con, world);

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
            Key {
                code: KeyCode::Char,
                shift: true,
                printable: '`',
                ..
            } => {
                let command_string = self.run(String::new(), CommandLine {});
                println!("ENGINE: $ {:?}", command_string);
                Some(Command(command_string))
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
