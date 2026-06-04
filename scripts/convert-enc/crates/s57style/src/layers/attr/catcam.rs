use crate::feature::{PropMap, PropMapExt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Catcam {
    NorthCardinalMark = 1,
    EastCardinalMark = 2,
    SouthCardinalMark = 3,
    WestCardinalMark = 4,
}

pub fn catcam(props: &PropMap) -> Option<Catcam> {
    match props.get_int("CATCAM")? {
        1 => Some(Catcam::NorthCardinalMark),
        2 => Some(Catcam::EastCardinalMark),
        3 => Some(Catcam::SouthCardinalMark),
        4 => Some(Catcam::WestCardinalMark),
        _ => None,
    }
}
