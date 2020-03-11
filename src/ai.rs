use crate::game::{self, Action, Game};
use crate::{rng, PLAYER};

#[derive(Debug)]
pub enum Ai {
    Basic,
    Idle,
    Confused { previous: Box<Ai>, num_turns: i32 },
}

/// Calculate an Ai turn
pub fn turn(id: usize, ai: Ai, game: &Game) -> (game::Turn, Ai) {
    // If you can see it, it can see you
    match ai {
        Ai::Basic => basic(id, &game),
        Ai::Idle => idle(id, &game),
        Ai::Confused {
            previous,
            num_turns,
        } => confused(id, &game, previous, num_turns),
    }
}

/// When the monster is confused
fn confused(_id: usize, _game: &Game, previous: Box<Ai>, num_turns: i32) -> (game::Turn, Ai) {
    let turn = vec![];
    let ai = if num_turns >= 1 {
        let num_turns = num_turns - 1;
        Ai::Confused {
            previous,
            num_turns,
        }
    } else {
        *previous
    };
    (turn, ai)
}

/// When the monster sees the player
fn basic(id: usize, game: &Game) -> (game::Turn, Ai) {
    let mut turn = vec![];
    let object = &game.objects[id];
    let player = &game.objects[PLAYER];

    if game.visible(&object.loc) {
        if game::distance(&object.loc, &player.loc) >= 2.0 {
            if rng::d12() > 11 {
                turn.push(Action::Bark(id));
            }
            turn.push(Action::Move(id, game::direction(&object.loc, &player.loc)));
            (turn, Ai::Basic)
        } else if player.fighter.map_or(false, |f| f.health > 0) {
            turn.push(Action::Attack(id, PLAYER));
            (turn, Ai::Basic)
        } else {
            (turn, Ai::Basic)
        }
    } else {
        (turn, Ai::Idle)
    }
}

/// When the monster does not see the player
fn idle(id: usize, game: &Game) -> (game::Turn, Ai) {
    let mut turn = vec![];
    let object = &game.objects[id];

    if game.visible(&object.loc) {
        (turn, Ai::Basic)
    } else if rng::dx(1000) > 999 {
        turn.push(Action::Mumble(id));
        (turn, Ai::Idle)
    } else {
        (turn, Ai::Idle)
    }
}
