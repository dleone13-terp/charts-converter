use crate::feature::{PropMap, PropMapExt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Catwrk {
    NonDangerous = 1,
    Dangerous = 2,
    Distributed = 3,
    ShowingMast = 4,
    ShowingHull = 5,
}

pub fn catwrk(props: &PropMap) -> Option<Catwrk> {
    match props.get_int("CATWRK")? {
        1 => Some(Catwrk::NonDangerous),
        2 => Some(Catwrk::Dangerous),
        3 => Some(Catwrk::Distributed),
        4 => Some(Catwrk::ShowingMast),
        5 => Some(Catwrk::ShowingHull),
        _ => None,
    }
}
