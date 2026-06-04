use crate::feature::{PropMap, PropMapExt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Convis {
    VisualConspicuous = 1,
    NotVisualConspicuous = 2,
}

pub fn convis(props: &PropMap) -> Option<Convis> {
    match props.get_int("CONVIS")? {
        1 => Some(Convis::VisualConspicuous),
        2 => Some(Convis::NotVisualConspicuous),
        _ => None,
    }
}
