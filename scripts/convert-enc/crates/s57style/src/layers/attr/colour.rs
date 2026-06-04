use crate::feature::{PropMap, PropMapExt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Colour {
    White = 1,
    Black = 2,
    Red = 3,
    Green = 4,
    Blue = 5,
    Yellow = 6,
    Grey = 7,
    Brown = 8,
    Amber = 9,
    Violet = 10,
    Orange = 11,
    Magenta = 12,
    Pink = 13,
}

fn int_to_colour(i: i64) -> Option<Colour> {
    match i {
        1 => Some(Colour::White),
        2 => Some(Colour::Black),
        3 => Some(Colour::Red),
        4 => Some(Colour::Green),
        5 => Some(Colour::Blue),
        6 => Some(Colour::Yellow),
        7 => Some(Colour::Grey),
        8 => Some(Colour::Brown),
        9 => Some(Colour::Amber),
        10 => Some(Colour::Violet),
        11 => Some(Colour::Orange),
        12 => Some(Colour::Magenta),
        13 => Some(Colour::Pink),
        _ => None,
    }
}

/// Read the COLOUR attribute as a list of colours.
pub fn colours(props: &PropMap) -> Vec<Colour> {
    props.get_int_list("COLOUR").into_iter().filter_map(int_to_colour).collect()
}

/// Return the first colour from the COLOUR attribute.
pub fn first_colour(props: &PropMap) -> Option<Colour> {
    props.get_int_list("COLOUR").into_iter().find_map(int_to_colour)
}
