use rand::Rng;

/// Random number within an inclusive [min:max] range
pub fn within(min: i32, max: i32) -> i32 {
    rand::thread_rng().gen_range(min, max + 1)
}

pub fn chance(p: f32) -> bool {
    rand::thread_rng().next_f32() <= p
}

pub fn choose<T>(values: &[T]) -> Option<&T> {
    rand::thread_rng().choose(values)
}

/// Roll custom dice
pub fn dx(x: i32) -> i32 {
    match x {
        0 => 0,
        x => rand::thread_rng().gen_range(1, x + 1),
    }
}
/// Roll n custom dice
pub fn ndx(n: i32, x: i32) -> i32 {
    (0..n).map(|_| dx(x)).sum()
}
/// Roll 1d3
pub fn d3() -> i32 {
    rand::thread_rng().gen_range(1, 4)
}
/// Roll nd3
pub fn nd3(n: i32) -> i32 {
    (0..n).map(|_| d3()).sum()
}
/// Roll 1d6
pub fn d6() -> i32 {
    rand::thread_rng().gen_range(1, 7)
}
/// Roll nd6
pub fn nd6(n: i32) -> i32 {
    (0..n).map(|_| d6()).sum()
}
/// Roll 1d12
pub fn d12() -> i32 {
    rand::thread_rng().gen_range(1, 13)
}
/// Roll nd12
pub fn nd12(n: i32) -> i32 {
    (0..n).map(|_| d12()).sum()
}
/// Roll 1d20
pub fn d20() -> i32 {
    rand::thread_rng().gen_range(1, 21)
}
/// Roll 1d100
pub fn d100() -> i32 {
    rand::thread_rng().gen_range(1, 101)
}
