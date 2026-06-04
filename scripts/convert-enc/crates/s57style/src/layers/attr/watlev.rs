use crate::feature::{PropMap, PropMapExt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Watlev {
    PartlySubmerged = 1,
    AlwaysDry = 2,
    AlwaysUnderwater = 3,
    CoversAndUncovers = 4,
    Awash = 5,
    SubjectToInundation = 6,
    Floating = 7,
}

pub fn watlev(props: &PropMap) -> Option<Watlev> {
    match props.get_int("WATLEV")? {
        1 => Some(Watlev::PartlySubmerged),
        2 => Some(Watlev::AlwaysDry),
        3 => Some(Watlev::AlwaysUnderwater),
        4 => Some(Watlev::CoversAndUncovers),
        5 => Some(Watlev::Awash),
        6 => Some(Watlev::SubjectToInundation),
        7 => Some(Watlev::Floating),
        _ => None,
    }
}
