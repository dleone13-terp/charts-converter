use crate::feature::{PropMap, PropMapExt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Restrn {
    AnchoringProhibited = 1,
    AnchoringRestricted = 2,
    FishingProhibited = 3,
    FishingRestricted = 4,
    TrawlingProhibited = 5,
    TrawlingRestricted = 6,
    EntryProhibited = 7,
    EntryRestricted = 8,
    DredgingProhibited = 9,
    DredgingRestricted = 10,
    DivingProhibited = 11,
    DivingRestricted = 12,
    NoWake = 13,
    AreaToBeAvoided = 14,
    ConstructionProhibited = 15,
}

pub fn restrn(props: &PropMap) -> Vec<Restrn> {
    props.get_int_list("RESTRN").into_iter().filter_map(|i| match i {
        1 => Some(Restrn::AnchoringProhibited),
        2 => Some(Restrn::AnchoringRestricted),
        3 => Some(Restrn::FishingProhibited),
        4 => Some(Restrn::FishingRestricted),
        5 => Some(Restrn::TrawlingProhibited),
        6 => Some(Restrn::TrawlingRestricted),
        7 => Some(Restrn::EntryProhibited),
        8 => Some(Restrn::EntryRestricted),
        9 => Some(Restrn::DredgingProhibited),
        10 => Some(Restrn::DredgingRestricted),
        11 => Some(Restrn::DivingProhibited),
        12 => Some(Restrn::DivingRestricted),
        13 => Some(Restrn::NoWake),
        14 => Some(Restrn::AreaToBeAvoided),
        15 => Some(Restrn::ConstructionProhibited),
        _ => None,
    }).collect()
}
