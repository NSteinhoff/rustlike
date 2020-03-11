// Stdlib
use std::cmp;

// Internal
use crate::{game, rng};
use crate::{PLAYER};
use crate::{Dimension, Location};
use crate::game::{Object, Map, Tile, Item};

/// Create a new map
pub fn make_map(
        objects: &mut Vec<Object>,
        map_dimension: Dimension,
        room_dimensions: Dimension,
        max_rooms: i32,
        max_room_monsters: i32,
        max_room_items: i32,
    ) -> Map {
    // fill map with "unblocked" tiles
    let Dimension(width, height) = map_dimension;
    let mut map = vec![vec![Tile::wall(); height as usize]; width as usize];
    let mut rooms: Vec<Rect> = vec![];

    let Dimension(min_room_size, max_room_size) = room_dimensions;
    for _ in 0..max_rooms {
        // random width and height
        let w = rng::within(min_room_size, max_room_size);
        let h = rng::within(min_room_size, max_room_size);
        // random position without going out of bounds
        let x = rng::within(0, width - w - 1);
        let y = rng::within(0, height - h - 1);

        let room = Rect::new(x, y, w, h);
        // check for intersections with exising rooms
        let intersects = rooms.iter().any(|other| room.intersects_with(other));

        if !intersects {
            create_room(room, &mut map);

            let (new_x, new_y) = room.center();
            if rooms.is_empty() {
                // put the player in the center of the first room
                objects[PLAYER].loc = Location(new_x, new_y);
            } else {
                // populate with some monsters
                place_objects(room, objects, max_room_monsters, max_room_items);
                // connect to the previous room
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                // toss a coin
                if rand::random() {
                    // first move horizontally, then vertically
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    // first move vertically, then horizontally
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }

            // Add this room to the list
            rooms.push(room);
        }
    }

    map
}

/// A rectangle on the map, used to characterise a room
#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }
    fn center(&self) -> (i32, i32) {
        let x = (self.x1 + self.x2) / 2;
        let y = (self.y1 + self.y2) / 2;
        (x, y)
    }
    fn intersects_with(&self, other: &Rect) -> bool {
        // returns true if this rectangle intersects another
        (self.x1 <= other.x2)
            && (self.x2 >= other.x1)
            && (self.y1 <= other.y2)
            && (self.y2 >= other.y1)
    }
}

/// Place a room on the map
fn create_room(room: Rect, map: &mut Map) {
    // go through the tiles in the rectangle and make them passable.
    // leave a one tile wide wall on the outside.
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

/// Create a horizontal tunnel
fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    // horizontal tunnel. `min()` and `mac()` are used in case `x1 > x2`
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

/// Create a vertical tunnel
fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    // vertical tunnel. `min()` and `max()` are used in case `y1 > y2`
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}


// -------------------------------- Monsters ----------------------------------

/// Return a random position inside a room
fn loc_in_room(room: Rect) -> Location {
    let x = rng::within(room.x1 + 1, room.x2 - 1);
    let y = rng::within(room.y1 + 1, room.y2 - 1);
    Location(x, y)
}

/// Create monster
fn create_monster(room: Rect) -> Object {
    let loc = loc_in_room(room);
    let roll = rng::d100();
    if roll < 50 {
        game::Object::orc(loc)
    } else if roll < 80 {
        game::Object::troll(loc)
    } else {
        game::Object::ogre(loc)
    }
}

/// Create item
fn create_item(room: Rect) -> Object {
    let loc = loc_in_room(room);
    let roll = rng::d100();
    if roll < 50 {
        game::Object::potion(loc, Item::Heal, "healing potion")
    } else {
        game::Object::scroll(loc, Item::Lightning, "lightning bolt")
    }
}

/// Place some monsters in random locations in a room
fn place_objects(room: Rect, objects: &mut Vec<Object>, max_room_monsters: i32, max_room_items: i32) {
    // choose a random number of monsters to place in this room
    for _ in 0..rng::within(0, max_room_monsters) {
        let monster = create_monster(room);

        // only place the monster, if the position isn't blocked yet
        if !game::object_blocks(&monster.loc, objects) {
            objects.push(monster);
        }
    }
    for _ in 0..rng::within(0, max_room_items) {
        let item = create_item(room);
        objects.push(item);
    }
}
