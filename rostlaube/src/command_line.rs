use crate::{State, Event, Transition};
use crate::colors;
use crate::console::{Console, Offscreen, TextAlignment, BackgroundFlag};
use crate::input::{Key, KeyCode};


#[derive(Debug)]
pub struct CommandLine;
#[derive(Debug)]
pub enum CommandLineAction {
    Confirm,
    Read(String),
    Delete,
    InvalidKey,
}

impl State for CommandLine {
    type World = String;
    type Action = CommandLineAction;

    fn render(&self, con: &mut Offscreen, world: &Self::World) {
        con.set_default_background(colors::BLUE);
        con.set_default_foreground(colors::WHITE);
        con.print_ex(
            0,
            0,
            BackgroundFlag::Set,
            TextAlignment::Left,
            format!("$ {}", world),
        );
    }

    fn interpret(
        &self,
        event: &Event,
    ) -> Self::Action {
        use CommandLineAction::*;
        use Event::*;
        match event {
            KeyEvent(Key {
                code: KeyCode::Enter,
                ..
            }) => Confirm,
            KeyEvent(Key {
                code: KeyCode::Backspace,
                ..
            }) => Delete,
            KeyEvent(Key {
                code: KeyCode::Spacebar,
                ..
            }) => Read(String::from(" ")),
            KeyEvent(Key {
                code: KeyCode::Char,
                printable,
                shift,
                ..
            }) => {
                if *shift {
                    Read(printable.to_uppercase().to_string())
                } else {
                    Read(printable.to_string())
                }
            }
            _ => InvalidKey,
        }
    }

    fn update(&mut self, action: Self::Action, world: &mut Self::World) -> Transition<Self> {
        use CommandLineAction::*;
        match action {
            Confirm => Transition::Exit,
            Read(s) => {
                world.push_str(&s);
                Transition::Continue
            }
            Delete => {
                world.pop();
                Transition::Continue
            }
            InvalidKey => Transition::Continue,
        }
    }
}
