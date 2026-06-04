use crate::feature::{PropMap, PropMapExt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Quasou {
    DepthKnown = 1,
    DepthUnknown = 2,
    Doubtful = 3,
    Unreliable = 4,
    NoBottomFound = 5,
    LeastDepthKnown = 6,
    LeastDepthUnknown = 7,
    ReportedNotSurveyed = 8,
    ReportedNotConfirmed = 9,
    MaintainedDepth = 10,
    NotRegularlyMaintained = 11,
}

pub fn quasou(props: &PropMap) -> Vec<Quasou> {
    props.get_int_list("QUASOU").into_iter().filter_map(|i| match i {
        1 => Some(Quasou::DepthKnown),
        2 => Some(Quasou::DepthUnknown),
        3 => Some(Quasou::Doubtful),
        4 => Some(Quasou::Unreliable),
        5 => Some(Quasou::NoBottomFound),
        6 => Some(Quasou::LeastDepthKnown),
        7 => Some(Quasou::LeastDepthUnknown),
        8 => Some(Quasou::ReportedNotSurveyed),
        9 => Some(Quasou::ReportedNotConfirmed),
        10 => Some(Quasou::MaintainedDepth),
        11 => Some(Quasou::NotRegularlyMaintained),
        _ => None,
    }).collect()
}

/// Returns true if the quality of sounding indicates a usable depth value.
pub fn is_usable_depth(quasou_list: &[Quasou]) -> bool {
    quasou_list.iter().any(|q| matches!(
        q,
        Quasou::DepthKnown
            | Quasou::NoBottomFound
            | Quasou::LeastDepthKnown
            | Quasou::LeastDepthUnknown
            | Quasou::MaintainedDepth
    ))
}
