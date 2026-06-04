use crate::feature::{PropMap, PropMapExt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Catlam {
    PortHand = 1,
    StarboardHand = 2,
    PreferredToStarboard = 3,
    PreferredToPort = 4,
}

pub fn catlam(props: &PropMap) -> Option<Catlam> {
    match props.get_int("CATLAM")? {
        1 => Some(Catlam::PortHand),
        2 => Some(Catlam::StarboardHand),
        3 => Some(Catlam::PreferredToStarboard),
        4 => Some(Catlam::PreferredToPort),
        _ => None,
    }
}
