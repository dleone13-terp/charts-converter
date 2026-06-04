use crate::feature::{PropMap, PropMapExt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bcnshp {
    StakePolePerchPost = 1,
    Whity = 2,
    BeaconTower = 3,
    LatticeBeacon = 4,
    PileBeacon = 5,
    Cairn = 6,
    BuoyantBeacon = 7,
}

pub fn bcnshp(props: &PropMap) -> Option<Bcnshp> {
    match props.get_int("BCNSHP")? {
        1 => Some(Bcnshp::StakePolePerchPost),
        2 => Some(Bcnshp::Whity),
        3 => Some(Bcnshp::BeaconTower),
        4 => Some(Bcnshp::LatticeBeacon),
        5 => Some(Bcnshp::PileBeacon),
        6 => Some(Bcnshp::Cairn),
        7 => Some(Bcnshp::BuoyantBeacon),
        _ => None,
    }
}
