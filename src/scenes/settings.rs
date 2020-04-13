use super::*;

#[derive(Debug)]
pub enum GameSettings {
    NewGame { player_name: String },
    LoadGame { path: String },
}

#[derive(Debug)]
pub enum Screen {
    MainMenu { player_name: String },
}

#[derive(Debug)]
pub enum Action {
    Cancel,
    StartGame,
    ReadChar(char, bool),
    DeleteChar,
    InvalidKey,
}

impl State for Screen {
    type World = Option<GameSettings>;
    type Action = Action;

    fn render(&self, con: &mut Offscreen, _settings: &Self::World) {
        use Screen::*;

        match self {
            MainMenu { player_name, .. } => {
                con.set_default_background(colors::BLACK);
                con.set_default_foreground(colors::WHITE);

                let (w, h) = (con.width(), con.height());

                let text = format!(
                    "{}\n\n{}\n\n\n\n\n{}",
                    "* Rustlike *",
                    "A short adventure in game development.",
                    "Press Enter to start a game. ESC to exit.",
                );

                con.print_rect_ex(
                    w / 2,
                    h / 4,
                    w - 2,
                    h - 2,
                    BackgroundFlag::Set,
                    TextAlignment::Center,
                    &text,
                );

                let num_lines_intro = con.get_height_rect(w / 2, h / 4, w - 2, h - 2, &text);

                con.print_ex(
                    w / 2,
                    h / 4 + num_lines_intro + 3,
                    BackgroundFlag::Set,
                    TextAlignment::Center,
                    format!("Enter name:\n{}", player_name),
                );
            }
        }
    }

    fn interpret(&self, event: &Event) -> Self::Action {
        use Action::*;
        use Event::*;
        use KeyCode::{Backspace, Char, Enter, Escape, Spacebar};
        use Screen::*;

        match self {
            MainMenu { .. } => match event {
                KeyEvent(Key { code: Escape, .. }) => Cancel,
                KeyEvent(Key { code: Enter, .. }) => StartGame,
                KeyEvent(Key {
                    code: Backspace, ..
                }) => DeleteChar,
                KeyEvent(Key {
                    code: Spacebar,
                    printable,
                    ..
                }) => ReadChar(*printable, false),
                KeyEvent(Key {
                    code: Char,
                    printable,
                    shift,
                    ..
                }) => ReadChar(*printable, *shift),
                Command(c) => {
                    println!("Execute {:?}", c);
                    InvalidKey
                }
                _ => InvalidKey,
            },
        }
    }

    fn update(&mut self, action: Self::Action, settings: &mut Self::World) -> Transition<Self> {
        use Action::*;
        use Screen::*;
        use Transition::*;

        match self {
            MainMenu { player_name, .. } => match action {
                StartGame => {
                    settings.replace(GameSettings::NewGame {
                        player_name: player_name.clone(),
                    });
                    Exit
                }
                DeleteChar => {
                    player_name.pop();
                    Continue
                }
                ReadChar(c, upper) => {
                    if upper {
                        for u in c.to_uppercase() {
                            player_name.push(u);
                        }
                    } else {
                        player_name.push(c);
                    }
                    Continue
                }
                Cancel => Exit,
                InvalidKey => Continue,
            },
        }
    }
}
