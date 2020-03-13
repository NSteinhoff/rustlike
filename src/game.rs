use std::cmp;

use crate::ai::{self, Ai};
use crate::engine::{colors, Color, Command, Engine, FovAlgorithm, FovMap};
use crate::{dungeon, rng, Dimension, Direction, Location, PLAYER};

/// Field of view algorithm
const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
/// FOV lights walls or not
const FOV_LIGHT_WALLS: bool = true;
/// FOV/torch radius
pub const TORCH_RADIUS: i32 = 10;
/// Healing potion amount of healing
const HEAL_AMOUNT: i32 = 10;
/// Range of the lightning bolt scroll
const LIGHTNING_RANGE: i32 = 3;
/// Damage of the lightning bolt scroll
const LIGHTNING_DAMAGE: i32 = 10;
/// Range of the consuse scroll
const CONFUSE_RANGE: i32 = 5;
/// The number of turns a monster is confused
const CONFUSE_NUM_TURNS: i32 = 5;

pub type Map = Vec<Vec<Tile>>;
pub type Turn = Vec<Action>;
pub type Message = (String, Color);
pub type Inventory = Vec<Object>;

/// Struct for tracking the game state
///
/// The game contains the `Map` and all objects.
pub struct Game {
    pub map: Map,
    pub objects: Vec<Object>,
    pub turn: i32,
    pub turns: Vec<(Turn, Turn)>,
    pub commands: Vec<Command>,
    pub messages: Messages,
    pub inventory: Inventory,
    pub fov: FovMap,
    pub map_dimensions: Dimension,
    pub player_turn: Turn,
}

impl Game {
    pub fn new(
        player: Object,
        map_dimensions: Dimension,
        room_dimensions: Dimension,
        max_rooms: i32,
        max_room_monsters: i32,
        max_room_items: i32,
    ) -> Self {
        let mut objects = vec![player];
        let Dimension(map_width, map_height) = map_dimensions;
        let mut game = Game {
            map: dungeon::make_map(
                &mut objects,
                map_dimensions,
                room_dimensions,
                max_rooms,
                max_room_monsters,
                max_room_items,
            ),
            objects: objects,
            turn: 0,
            turns: vec![],
            commands: vec![],
            messages: Messages::empty(),
            inventory: vec![],
            fov: FovMap::new(map_width, map_height),
            map_dimensions: map_dimensions,
            player_turn: vec![],
        };
        game.init_fov();

        game
    }

    pub fn turn(&mut self, player: Turn, ai: Turn) {
        self.turns.push((player, ai));
        self.turn += 1;
        self.player_turn.clear();
    }

    pub fn play(&mut self, turn: &Turn) {
        for action in turn {
            let msgs = match *action {
                Action::Move(id, direction) => {
                    move_object(id, direction, &self.map, &mut self.objects)
                }
                Action::Attack(id, target) => attack(id, target, &mut self.objects),
                Action::PickUp(id, target) => {
                    pickup_item(id, target, &mut self.objects, &mut self.inventory)
                }
                Action::Bark(id) => bark(id, &self.objects),
                Action::Mumble(id) => mumble(id, &self.objects),
                Action::Wait(_) => Messages::empty(),
                Action::UseItem(id, item) => use_item(id, item, self),
            };
            self.messages.append(msgs);
        }
    }

    /// Return player turn based on command input
    pub fn player_turn(
        &self,
        command: &Command,
        engine: &mut Engine,
    ) -> (Option<Action>, Messages) {
        use Command::*;
        if self.objects[PLAYER].alive {
            match command {
                // Movement
                Up => move_or_attack(PLAYER, Direction(0, -1), &self.map, &self.objects),
                Down => move_or_attack(PLAYER, Direction(0, 1), &self.map, &self.objects),
                Left => move_or_attack(PLAYER, Direction(-1, 0), &self.map, &self.objects),
                Right => move_or_attack(PLAYER, Direction(1, 0), &self.map, &self.objects),
                UpLeft => move_or_attack(PLAYER, Direction(-1, -1), &self.map, &self.objects),
                UpRight => move_or_attack(PLAYER, Direction(1, -1), &self.map, &self.objects),
                DownLeft => move_or_attack(PLAYER, Direction(-1, 1), &self.map, &self.objects),
                DownRight => move_or_attack(PLAYER, Direction(1, 1), &self.map, &self.objects),

                // Actions
                Grab => grab(PLAYER, &self.objects),
                Skip => (Some(Action::Wait(PLAYER)), Messages::empty()),

                OpenInventory => {
                    let mut items: Vec<&str> = vec![];
                    for item in &self.inventory {
                        items.push(&item.name);
                    }
                    if let Some(choice) = self.open_inventory(engine, "Pick item to use:") {
                        (Some(Action::UseItem(PLAYER, choice)), Messages::empty())
                    } else {
                        (None, Messages::empty())
                    }
                }

                // Unmapped command
                _ => (None, Messages::empty()),
            }
        } else {
            (None, Messages::empty())
        }
    }

    /// Monster turn
    pub fn ai_turns(&mut self) -> Turn {
        let mut actions = vec![];
        for id in PLAYER + 1..self.objects.len() {
            if let Some(ai) = self.objects[id].ai.take() {
                let (turn, new_ai) = ai::turn(id, ai, self);
                for a in turn {
                    actions.push(a);
                }
                self.objects[id].ai = Some(new_ai);
            }
        }
        actions
    }

    pub fn update(&mut self, full_turn: bool) -> Messages {
        let mut messages = Messages::empty();
        messages.append(self.update_fov());
        messages.append(self.update_map());
        messages.append(self.update_objects(full_turn));
        messages
    }

    fn update_map(&mut self) -> Messages {
        let Dimension(width, height) = self.map_dimensions;
        for y in 0..height {
            for x in 0..width {
                let visible = self.visible(&Location(x, y));
                let tile = &mut self.map[x as usize][y as usize];
                if visible {
                    tile.explored = true;
                    tile.visible = true;
                } else {
                    tile.visible = false;
                }
            }
        }
        Messages::empty()
    }

    fn update_objects(&mut self, full_turn: bool) -> Messages {
        let mut messages = Messages::empty();
        for id in 0..self.objects.len() {
            if self.visible(&self.objects[id].loc) {
                self.objects[id].visible = true;
                if !self.objects[id].seen {
                    messages.add(
                        format!("You see {}", indirect(&self.objects[id].name, false),),
                        colors::WHITE,
                    );
                    self.objects[id].seen = true;
                }
            } else {
                self.objects[id].visible = false;
            }

            if let Some(fighter) = self.objects[id].fighter {
                if fighter.health <= 0 {
                    let death_messages = fighter.on_death.call(&mut self.objects[id]);
                    messages.append(death_messages);
                }
            }

            if full_turn {
                let _ = regenerate(&mut self.objects[id]);
            }
        }
        messages
    }

    fn init_fov(&mut self) {
        let Dimension(width, height) = self.map_dimensions;
        for x in 0..width {
            for y in 0..height {
                self.fov.set(
                    x,
                    y,
                    !self.map[x as usize][y as usize].block_sight,
                    !self.map[x as usize][y as usize].blocked,
                )
            }
        }
    }

    fn update_fov(&mut self) -> Messages {
        let Location(x, y) = self.objects[PLAYER].loc;
        self.fov
            .compute_fov(x, y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
        Messages::empty()
    }

    pub fn visible(&self, loc: &Location) -> bool {
        let Location(x, y) = *loc;
        self.fov.is_in_fov(x, y)
    }

    fn open_inventory(&self, engine: &mut Engine, title: &str) -> Option<usize> {
        let mut items: Vec<&str> = vec![];
        for item in &self.inventory {
            items.push(&item.name);
        }
        engine.menu(title, &items, 25)
    }
}

pub struct Messages {
    messages: Vec<Message>,
}

impl Messages {
    pub fn empty() -> Self {
        Self { messages: vec![] }
    }

    pub fn new<T: Into<String>>(message: T, color: Color) -> Self {
        let mut messages = Self::empty();
        messages.add(message, color);
        messages
    }

    pub fn add<T: Into<String>>(&mut self, message: T, color: Color) {
        self.messages.push((message.into(), color));
    }

    pub fn append(&mut self, other: Self) {
        for (msg, color) in other.iter() {
            self.messages.push((msg.into(), *color));
        }
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &(String, Color)> {
        self.messages.iter()
    }
}

// --------------------------------- Objects ----------------------------------

/// A tile of the map and its properties
#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub blocked: bool,
    pub block_sight: bool,
    pub char: char,
    pub explored: bool,
    pub visible: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
            char: '.',
            explored: false,
            visible: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
            char: '#',
            explored: false,
            visible: false,
        }
    }
}

/// Generic object: the player, a monster, an item, the stairs...
/// It's always represented by a character on screen.
#[derive(Debug, Default)]
pub struct Object {
    pub loc: Location,
    pub char: char,
    pub color: Color,
    pub name: String,

    // Flags
    pub blocks: bool,
    pub visible: bool,
    pub seen: bool,
    pub alive: bool,

    // Components
    pub movement: Option<Movement>,
    pub fighter: Option<Fighter>,
    pub ai: Option<Ai>,
    pub noise: Option<Noise>,
    pub item: Option<Item>,
}

impl Object {
    pub fn new() -> Self {
        let mut o: Object = Default::default();
        o.char = '`';
        o.name = "it".into();
        o
    }
    pub fn player(loc: Location, name: &str) -> Self {
        let mut this = Object::new();
        this.loc = loc;
        this.name = String::from(name);
        this.char = '@';
        this.color = colors::YELLOW;

        this.blocks = true;
        this.alive = true;
        this.visible = true;
        this.seen = true;

        this.movement = Some(Movement { speed: 100 });
        this.fighter = Some(Fighter {
            max_health: 30,
            health: 30,
            defense: 2,
            power: 5,
            on_death: DeathCallback::Player,
            health_regen: 0.5,
        });

        this
    }
    pub fn orc(loc: Location) -> Self {
        let mut this = Object::new();
        this.loc = loc;
        this.name = String::from("orc");
        this.char = 'o';
        this.color = colors::GREEN;
        this.blocks = true;
        this.alive = true;

        this.ai = Some(Ai::Basic);
        this.movement = Some(Movement { speed: 90 });
        this.fighter = Some(Fighter {
            max_health: 10,
            health: 10,
            defense: 0,
            power: 3,
            on_death: DeathCallback::Monster,
            health_regen: 0.1,
        });
        this.noise = Some(Noise {
            bark: String::from("shout"),
            mumble: String::from("mumble"),
        });

        this
    }
    pub fn troll(loc: Location) -> Self {
        let mut this = Object::new();
        this.loc = loc;
        this.name = String::from("troll");
        this.char = 'T';
        this.color = colors::GREEN;
        this.blocks = true;
        this.alive = true;

        this.ai = Some(Ai::Basic);
        this.movement = Some(Movement { speed: 80 });
        this.fighter = Some(Fighter {
            max_health: 16,
            health: 16,
            defense: 1,
            power: 4,
            on_death: DeathCallback::Monster,
            health_regen: 0.5,
        });
        this.noise = Some(Noise {
            bark: String::from("roar"),
            mumble: String::from("growl"),
        });

        this
    }
    pub fn ogre(loc: Location) -> Self {
        let mut this = Object::new();
        this.loc = loc;
        this.name = String::from("ogre");
        this.char = 'O';
        this.color = colors::YELLOW;
        this.blocks = true;
        this.alive = true;

        this.ai = Some(Ai::Basic);
        this.movement = Some(Movement { speed: 70 });
        this.fighter = Some(Fighter {
            max_health: 25,
            health: 25,
            defense: 2,
            power: 8,
            on_death: DeathCallback::Monster,
            health_regen: 0.2,
        });
        this.noise = Some(Noise {
            bark: String::from("bellow"),
            mumble: String::from("burp"),
        });

        this
    }
    pub fn potion<T: Into<String>>(loc: Location, item: Item, name: T) -> Self {
        let mut this = Object::new();
        this.loc = loc;
        this.name = name.into();
        this.char = '!';
        this.color = colors::BLUE;
        this.item = Some(item);

        this
    }
    pub fn scroll<T: Into<String>>(loc: Location, item: Item, name: T) -> Self {
        let mut this = Object::new();
        this.loc = loc;
        this.name = name.into();
        this.char = '?';
        this.color = colors::BLUE;
        this.item = Some(item);

        this
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Fighter {
    pub max_health: i32,
    pub health: i32,
    pub defense: i32,
    pub power: i32,
    pub on_death: DeathCallback,
    pub health_regen: f32,
}

impl Fighter {
    fn take_damage(&mut self, damage: i32) {
        self.health -= damage;
    }
    fn heal(&mut self, amount: i32) {
        self.health = cmp::min(self.health + amount, self.max_health);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DeathCallback {
    Player,
    Monster,
}

impl DeathCallback {
    fn call(&self, object: &mut Object) -> Messages {
        use DeathCallback::*;
        match self {
            Player => kill_player(object),
            Monster => kill_monster(object),
        }
    }
}

#[derive(Debug)]
pub struct Noise {
    pub bark: String,
    pub mumble: String,
}

#[derive(Debug)]
pub struct Movement {
    pub speed: i32,
}

#[derive(Debug)]
pub enum Item {
    Heal,
    Lightning,
    Confusion,
}

// --------------------------------- Actions ----------------------------------

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Move(usize, Direction),
    Attack(usize, usize),
    PickUp(usize, usize),
    UseItem(usize, usize),
    Bark(usize),
    Mumble(usize),
    Wait(usize),
}

impl Action {
    pub fn took_turn(&self) -> bool {
        use Action::*;
        match self {
            Move(_, _) => true,
            Attack(_, _) => true,
            PickUp(_, _) => true,
            UseItem(_, _) => false,
            Bark(_) => true,
            Mumble(_) => true,
            Wait(_) => true,
        }
    }
}

/// Pick a move or attack action
pub fn move_or_attack(
    id: usize,
    direction: Direction,
    map: &Map,
    objects: &[Object],
) -> (Option<Action>, Messages) {
    let destination = destination(&objects[id].loc, &direction);
    if object_blocks(&destination, objects) {
        match objects
            .iter()
            .position(|o| o.loc == destination && o.fighter.is_some())
        {
            Some(defender) => (Some(Action::Attack(id, defender)), Messages::empty()),
            None => (None, Messages::new("Cannot attack that.", colors::WHITE)),
        }
    } else if structure_blocks(&destination, map) {
        (None, Messages::new("It's blocked.", colors::WHITE))
    } else {
        (Some(Action::Move(id, direction)), Messages::empty())
    }
}

/// Grab an object
pub fn grab(id: usize, objects: &[Object]) -> (Option<Action>, Messages) {
    let item_id = objects
        .iter()
        .position(|o| o.loc == objects[id].loc && o.item.is_some());

    if let Some(item_id) = item_id {
        (Some(Action::PickUp(id, item_id)), Messages::empty())
    } else {
        (
            None,
            Messages::new("There is nothing here to pick up.", colors::WHITE),
        )
    }
}

// ------------------------------- Resolution ---------------------------------

enum UseResult {
    UsedUp,
    Cancelled,
}

/// Attack resolution
fn attack(attacker: usize, defender: usize, objects: &mut [Object]) -> Messages {
    let mut messages = Messages::empty();
    let damage = objects[attacker].fighter.map_or(0, |f| rng::dx(f.power))
        - objects[defender].fighter.map_or(0, |f| rng::dx(f.defense));

    if let Some(fighter) = objects[defender].fighter.as_mut() {
        let msg = match (attacker, defender) {
            (PLAYER, d) => format!("You attack {}", direct(&objects[d].name, false)),
            (a, PLAYER) => format!("{} attacks you", direct(&objects[a].name, true)),
            (a, d) => format!(
                "{} attacks {}",
                direct(&objects[a].name, true),
                direct(&objects[d].name, false)
            ),
        };
        if damage > 0 {
            let msg = format!("{} for {} damage!", msg, damage);
            messages.add(msg, colors::WHITE);
            fighter.take_damage(damage);
        } else {
            let msg = match attacker {
                PLAYER => format!("{} but do no damage.", msg),
                _ => format!("{} but does no damage.", msg),
            };
            messages.add(msg, colors::WHITE);
        }
    }
    messages
}

/// Move resolution
fn move_object(id: usize, direction: Direction, map: &Map, objects: &mut [Object]) -> Messages {
    let Direction(dx, dy) = direction;
    let mut messages = Messages::empty();
    if objects[id]
        .movement
        .as_ref()
        .map_or(false, |m| m.speed >= rng::d100())
    {
        let _ = move_by(id, direction, map, objects)
            || move_by(id, Direction(dx, 0), map, objects)
            || move_by(id, Direction(0, dy), map, objects);
    } else {
        messages.add("The way is blocked!", colors::WHITE);
    };
    messages
}

/// Pick up item
fn pickup_item(
    actor: usize,
    item_id: usize,
    objects: &mut Vec<Object>,
    inventory: &mut Inventory,
) -> Messages {
    let mut messages = Messages::empty();
    if inventory.len() >= 26 {
        messages.add("Inventory full", colors::WHITE);
    } else {
        let item = objects.swap_remove(item_id);

        let msg = match actor {
            PLAYER => format!("You pick up {}.", indirect(&item.name, false)),
            _ => format!(
                "{} picks up {}.",
                direct(&objects[actor].name, true),
                indirect(&item.name, false)
            ),
        };
        messages.add(msg, colors::WHITE);

        inventory.push(item);
    }
    messages
}

/// Use an item
fn use_item(id: usize, item_id: usize, game: &mut Game) -> Messages {
    use Item::*;
    if let Some(item) = &game.inventory[item_id].item {
        let on_use = match item {
            Heal => cast_heal,
            Lightning => cast_lightning,
            Confusion => cast_confusion,
        };
        match on_use(id, item_id, game) {
            (UseResult::UsedUp, messages) => {
                game.inventory.remove(item_id);
                messages
            }
            (UseResult::Cancelled, messages) => messages,
        }
    } else {
        Messages::empty()
    }
}

fn bark(id: usize, objects: &[Object]) -> Messages {
    let mut messages = Messages::empty();
    let o = &objects[id];
    if let Some(Noise { bark, .. }) = &o.noise {
        messages.add(
            format!("{} {}s.", indirect(&o.name, true), bark),
            colors::WHITE,
        );
    }
    messages
}

fn mumble(id: usize, objects: &[Object]) -> Messages {
    let mut messages = Messages::empty();
    let o = &objects[id];
    if let Some(Noise { mumble, .. }) = &o.noise {
        messages.add(
            format!("{} {}s.", direct(&o.name, true), mumble),
            colors::WHITE,
        );
    }
    messages
}

fn kill_player(player: &mut Object) -> Messages {
    let mut messages = Messages::empty();
    let msg = "You die!";
    player.alive = false;
    player.char = '%';
    player.color = colors::RED;

    messages.add(msg, colors::RED);
    messages
}

fn kill_monster(monster: &mut Object) -> Messages {
    let mut messages = Messages::empty();
    monster.alive = false;
    let msg = format!("{} dies.", direct(&monster.name, true));

    monster.char = '%';
    monster.color = colors::RED;
    monster.blocks = false;
    monster.fighter = None;
    monster.ai = None;
    monster.name = format!("Remains of {}", monster.name);

    messages.add(msg, colors::RED);
    messages
}

fn regenerate(object: &mut Object) -> Messages {
    if let (true, Some(fighter)) = (object.alive, object.fighter.as_mut()) {
        let amount = match fighter.health_regen {
            p if p <= 1.0 => rng::chance(p) as i32,
            v => v as i32,
        };
        fighter.heal(amount);
    }
    Messages::empty()
}

// --------------------------------- Movement ----------------------------------
/// Distance between two points
pub fn distance(a: &Location, b: &Location) -> f32 {
    let Location(ax, ay) = a;
    let Location(bx, by) = b;
    let (dx, dy) = (bx - ax, by - ay);
    ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
}

/// Calculate normalized direction between two points
pub fn direction(a: &Location, b: &Location) -> Direction {
    let Location(ax, ay) = a;
    let Location(bx, by) = b;

    let dx = match bx - ax {
        x if x < 0 => -1,
        x if x > 0 => 1,
        _ => 0,
    };
    let dy = match by - ay {
        y if y < 0 => -1,
        y if y > 0 => 1,
        _ => 0,
    };
    Direction(dx, dy)
}

/// Get the destination when moving from a location in a given direction
fn destination(location: &Location, direction: &Direction) -> Location {
    let Location(x, y) = location;
    let Direction(dx, dy) = direction;
    Location(x + dx, y + dy)
}

/// Move by the given amount
fn move_by(id: usize, direction: Direction, map: &Map, objects: &mut [Object]) -> bool {
    let destination = destination(&objects[id].loc, &direction);
    if !(structure_blocks(&destination, map) || object_blocks(&destination, objects)) {
        objects[id].loc = destination;
        true
    } else {
        false
    }
}

// -------------------------------- Collision ---------------------------------
/// Check if and object is at this position
pub fn object_blocks(loc: &Location, objects: &[Object]) -> bool {
    objects
        .iter()
        .any(|object| object.blocks && &object.loc == loc)
}

/// Check if a structure blocks at this position
fn structure_blocks(loc: &Location, map: &Map) -> bool {
    let Location(x, y) = *loc;
    map[x as usize][y as usize].blocked
}

/// Find the closest fighter within range
pub fn fighters_by_distance(id: usize, objects: &[Object], range: i32) -> Vec<usize> {
    let loc = &objects[id].loc;
    let mut in_range: Vec<(i32, usize)> = objects
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != id) // don't target yourself
        .filter(|(_, o)| o.fighter.is_some()) // only target fighters
        .map(|(i, o)| (distance(loc, &o.loc) as i32, i)) // get the distance
        .filter(|&(d, _)| d <= range) // only targets in range
        .collect(); // collect into a vector to enable sorting
    in_range.sort_by_key(|(d, _)| -d); // descending sort by distance
    in_range.iter().map(|(_, i)| i).cloned().collect()
}

/// Find the closest fighter within range
fn closest_fighter(id: usize, objects: &[Object], range: i32) -> Option<usize> {
    fighters_by_distance(id, objects, range).pop()
}

/// Find a random fighter within range
fn random_fighter(id: usize, objects: &[Object], range: i32) -> Option<usize> {
    let loc = &objects[id].loc;
    let target = objects
        .iter()
        .enumerate()
        .map(|(i, o)| (i, &o.loc))
        .position(|(i, l)| i != id && distance(loc, l) <= range as f32);

    target.filter(|&t| objects[t].fighter.is_some())
}

/// Check if a place on the map is blocked
fn is_blocked(loc: &Location, map: &Map, objects: &[Object]) -> bool {
    structure_blocks(loc, map) || object_blocks(loc, objects)
}

fn indirect(it: &str, upper: bool) -> String {
    let an = "aeiou".chars().find(|&c| it.starts_with(c)).is_some();

    let article = match (upper, an) {
        (true, true) => "An",
        (false, true) => "an",
        (true, false) => "A",
        (false, false) => "a",
    };
    format!("{} {}", article, it)
}

fn direct(it: &str, upper: bool) -> String {
    let article = if upper { "The" } else { "the" };
    format!("{} {}", article, it)
}

// --------------------------- Items and Abilities ----------------------------
fn cast_heal(id: usize, _item_id: usize, game: &mut Game) -> (UseResult, Messages) {
    let o = &mut game.objects[id];
    if let Some(fighter) = o.fighter.as_mut() {
        if fighter.health == fighter.max_health {
            (
                UseResult::Cancelled,
                Messages::new("Already at full health!", colors::WHITE),
            )
        } else {
            fighter.heal(HEAL_AMOUNT);
            (UseResult::UsedUp, Messages::new("Healed!", colors::WHITE))
        }
    } else {
        (
            UseResult::Cancelled,
            Messages::new("Only fighters can drink!", colors::WHITE),
        )
    }
}
fn cast_lightning(id: usize, _item_id: usize, game: &mut Game) -> (UseResult, Messages) {
    if let Some(target) = closest_fighter(id, &game.objects, LIGHTNING_RANGE) {
        let t = &mut game.objects[target];
        let mut f = t.fighter.as_mut().expect("Target must be a fighter");
        f.health -= LIGHTNING_DAMAGE;
        (
            UseResult::UsedUp,
            Messages::new(
                format!("You zap {} ", direct(&t.name, false)),
                colors::WHITE,
            ),
        )
    } else {
        (
            UseResult::Cancelled,
            Messages::new("There are no targets in range.", colors::WHITE),
        )
    }
}
fn cast_confusion(id: usize, _item_id: usize, game: &mut Game) -> (UseResult, Messages) {
    if let Some(target) = closest_fighter(id, &game.objects, CONFUSE_RANGE) {
        let t = &mut game.objects[target];
        if let Some(ai) = t.ai.take() {
            t.ai = Some(Ai::Confused {
                previous: Box::new(ai),
                num_turns: CONFUSE_NUM_TURNS,
            });
            (
                UseResult::UsedUp,
                Messages::new(
                    format!("{} looks confused.", direct(&t.name, true)),
                    colors::WHITE,
                ),
            )
        } else {
            (
                UseResult::Cancelled,
                Messages::new("There are no targets in range.", colors::WHITE),
            )
        }
    } else {
        (
            UseResult::Cancelled,
            Messages::new("There are no targets in range.", colors::WHITE),
        )
    }
}
