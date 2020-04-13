use super::*;

#[derive(Debug)]
pub enum Screen {
    GameWorld,
    Console,
    Inventory,
    Character,
}

#[derive(Debug)]
pub enum Action {
    Nothing,
    Exit,
    OpenInventory,
    OpenCharacterScreen,
    ListObjects,
    GameAction(game::Action),
}

impl State for Screen {
    type World = Game;
    type Action = Action;

    fn render(&self, con: &mut Offscreen, game: &Self::World) {
        use Screen::*;

        match self {
            GameWorld => {
                game.render_game_world(con);
                game.render_messages(con);
            }
            Inventory => println!("Show inventory"),
            Character => println!("Show character"),
            Console => println!("Show console"),
        };
    }

    fn interpret(&self, event: &Event) -> Self::Action {
        use Action::*;
        use Event::*;
        use KeyCode::{Char, Escape};
        use Screen::*;

        match self {
            GameWorld => match event {
                KeyEvent(Key { code: Escape, .. }) => Exit,
                KeyEvent(Key {
                    code: Char,
                    printable: 'i',
                    ..
                }) => OpenInventory,
                KeyEvent(Key {
                    code: Char,
                    printable: 'c',
                    ..
                }) => OpenCharacterScreen,
                KeyEvent(Key {
                    code: Char,
                    printable: c,
                    ..
                }) => game_action(c),
                KeyEvent(_) | Event::Nothing => Action::Nothing,
                Command(c) => execute(c),
            },
            Inventory => Exit,
            Character => Exit,
            Console => Exit,
        }
    }

    fn update(&mut self, action: Self::Action, game: &mut Self::World) -> Transition<Self> {
        use Action::*;
        use Screen::*;

        match self {
            GameWorld => match action {
                Exit => Transition::Exit,
                Nothing => Transition::Continue,
                OpenInventory => Transition::Next(Inventory),
                OpenCharacterScreen => Transition::Next(Character),
                GameAction(action) => {
                    game.update(action);
                    Transition::Continue
                },
                ListObjects => {
                    for (i, o) in game.objects.iter().enumerate() {
                        println!("{}: {:?}", i, o);
                    }
                    Transition::Continue
                }
            },
            Inventory => Transition::Exit,
            Character => Transition::Exit,
            Console => Transition::Exit,
        }
    }
}

fn game_action(c: &char) -> Action {
    use game::Action::*;
    let a = match c {
        'k' => Move(PLAYER, Direction(0, -1)),
        'j' => Move(PLAYER, Direction(0, 1)),
        'h' => Move(PLAYER, Direction(-1, 0)),
        'l' => Move(PLAYER, Direction(1, 0)),
        'y' => Move(PLAYER, Direction(-1, -1)),
        'u' => Move(PLAYER, Direction(1, -1)),
        'b' => Move(PLAYER, Direction(-1, 1)),
        'n' => Move(PLAYER, Direction(1, 1)),
        _ => game::Action::Nothing,
    };
    Action::GameAction(a)
}

fn execute(command: &str) -> Action {
    match command {
        "ls" => {
            println!("List objects");
            Action::ListObjects
        }
        _ => {
            println!("Unknown command: {:?}", command);
            Action::Nothing
        }
    }
}
