use crate::feature::{PropMap, PropMapExt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Catlmk {
    Cairn = 1,
    Cemetery = 2,
    Chimney = 3,
    DishAerial = 4,
    FlagstaffFlagpole = 5,
    FlareStack = 6,
    Mast = 7,
    Windsock = 8,
    Monument = 9,
    ColumnPillar = 10,
    MemorialPlaque = 11,
    Obelisk = 12,
    Statue = 13,
    Cross = 14,
    Dome = 15,
    RadarScanner = 16,
    Tower = 17,
    Windmill = 18,
    Windmotor = 19,
    SpireMinaret = 20,
}

pub fn catlmk(props: &PropMap) -> Vec<Catlmk> {
    props.get_int_list("CATLMK").into_iter().filter_map(|i| match i {
        1 => Some(Catlmk::Cairn),
        2 => Some(Catlmk::Cemetery),
        3 => Some(Catlmk::Chimney),
        4 => Some(Catlmk::DishAerial),
        5 => Some(Catlmk::FlagstaffFlagpole),
        6 => Some(Catlmk::FlareStack),
        7 => Some(Catlmk::Mast),
        8 => Some(Catlmk::Windsock),
        9 => Some(Catlmk::Monument),
        10 => Some(Catlmk::ColumnPillar),
        11 => Some(Catlmk::MemorialPlaque),
        12 => Some(Catlmk::Obelisk),
        13 => Some(Catlmk::Statue),
        14 => Some(Catlmk::Cross),
        15 => Some(Catlmk::Dome),
        16 => Some(Catlmk::RadarScanner),
        17 => Some(Catlmk::Tower),
        18 => Some(Catlmk::Windmill),
        19 => Some(Catlmk::Windmotor),
        20 => Some(Catlmk::SpireMinaret),
        _ => None,
    }).collect()
}
