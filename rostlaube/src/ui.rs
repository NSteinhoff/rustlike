use std::cmp;

use crate::colors::{self, Color};
use crate::console::{self, BackgroundFlag, Console, Offscreen, TextAlignment};
use crate::Location;

/// Draw an object on the view
pub trait Draw {
    fn draw(&self, layer: &mut Offscreen, loc: &Location);
}
pub fn draw(item: &impl Draw, layer: &mut Offscreen, loc: &Location) {
    item.draw(layer, loc)
}

#[derive(Debug)]
pub struct Bar {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub name: String,
    pub current: i32,
    pub maximum: i32,
    pub color: Color,
    pub background: Color,
}

impl Draw for Bar {
    fn draw(&self, layer: &mut Offscreen, _loc: &Location) {
        // Make sure we don't exceed the width of the console
        let width = cmp::min(layer.width(), self.width) - self.x;

        let mut con = Offscreen::new(width, 1);

        con.set_default_background(self.background);

        con.rect(0, 0, width, 1, false, BackgroundFlag::Set);

        con.set_default_background(self.color);
        let pct_filled = self.current as f32 / self.maximum as f32 * width as f32;
        let filled = pct_filled as i32;
        if filled > 0 {
            con.rect(0, 0, filled, 1, false, BackgroundFlag::Set);
        }

        con.set_default_foreground(colors::BLACK);
        con.print_ex(
            2, // Draw it on the right side of the bar
            0,
            BackgroundFlag::None,
            TextAlignment::Left,
            &format!("{}: {}/{}", self.name, self.current, self.maximum),
        );

        console::blit(&con, (0, 0), (width, 1), layer, (self.x, self.y), 1.0, 1.0);
    }
}
