use crate::feature::{PropMap, PropMapExt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Catobs {
    SnagStump = 1,
    Wellhead = 2,
    Diffuser = 3,
    Crib = 4,
    FishHaven = 5,
    FoulArea = 6,
    FoulGround = 7,
    IceBoom = 8,
    GroundTackle = 9,
    Boom = 10,
}

pub fn catobs(props: &PropMap) -> Option<Catobs> {
    match props.get_int("CATOBS")? {
        1 => Some(Catobs::SnagStump),
        2 => Some(Catobs::Wellhead),
        3 => Some(Catobs::Diffuser),
        4 => Some(Catobs::Crib),
        5 => Some(Catobs::FishHaven),
        6 => Some(Catobs::FoulArea),
        7 => Some(Catobs::FoulGround),
        8 => Some(Catobs::IceBoom),
        9 => Some(Catobs::GroundTackle),
        10 => Some(Catobs::Boom),
        _ => None,
    }
}
