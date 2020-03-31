//! Map geometry
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Location(pub i32, pub i32);
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Direction(pub i32, pub i32);
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Dimension(pub i32, pub i32);


pub fn translate(
    source: &Dimension,
    target: &Dimension,
    loc: &Location,
    focus: &Location,
) -> Option<Location> {
    let Dimension(width, height) = target;
    let Dimension(map_width, map_height) = source;

    let center_x = width / 2 + 1;
    let center_y = height / 2 + 1;

    let Location(x_focus, y_focus) = focus;
    let Location(x_map, y_map) = loc;

    let rel_x = x_map - x_focus;
    let rel_y = y_map - y_focus;

    let view_x = center_x + rel_x;
    let view_y = center_y + rel_y;

    if view_x >= 0 && view_x < *map_width && view_y >= 0 && view_y < *map_height {
        let view_loc = Location(view_x, view_y);
        Some(view_loc)
    } else {
        None
    }
}
