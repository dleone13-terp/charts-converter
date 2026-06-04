use crate::feature::{PropMap, PropMapExt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Boyshp {
    Conical = 1,
    Can = 2,
    Spherical = 3,
    Pillar = 4,
    Spar = 5,
    Barrel = 6,
    SuperBuoy = 7,
    IceBuoy = 8,
}

pub fn boyshp(props: &PropMap) -> Option<Boyshp> {
    match props.get_int("BOYSHP")? {
        1 => Some(Boyshp::Conical),
        2 => Some(Boyshp::Can),
        3 => Some(Boyshp::Spherical),
        4 => Some(Boyshp::Pillar),
        5 => Some(Boyshp::Spar),
        6 => Some(Boyshp::Barrel),
        7 => Some(Boyshp::SuperBuoy),
        8 => Some(Boyshp::IceBuoy),
        _ => None,
    }
}
