use crate::config::StyleConfig;
use crate::feature::PropMap;
use crate::style::FeatureStyle;
use super::attr::catlam::{catlam, Catlam};
use super::attr::boyshp::{boyshp, Boyshp};
use super::attr::colour::{first_colour, Colour};

fn half_triangle(props: &PropMap) -> &'static str {
    match first_colour(props) {
        Some(Colour::Red) => "BOYLAT14",
        Some(Colour::Green) => "BOYLAT13",
        _ => "BOYSPP15",
    }
}

fn rhomboid(props: &PropMap) -> &'static str {
    match first_colour(props) {
        Some(Colour::Red) => "BOYLAT24",
        Some(Colour::Green) => "BOYLAT23",
        _ => "BOYSPP25",
    }
}

fn circle(props: &PropMap) -> &'static str {
    match first_colour(props) {
        Some(Colour::Red) => "BOYSAW12",
        _ => "BOYSPP11",
    }
}

fn stake(props: &PropMap) -> &'static str {
    match first_colour(props) {
        Some(Colour::Red) => "BCNLAT21",
        Some(Colour::Green) => "BCNLAT22",
        Some(Colour::Black) => "BCNSAW21",
        _ => "BCNSPP21",
    }
}

fn by_boyshp(props: &PropMap) -> &'static str {
    match boyshp(props) {
        Some(Boyshp::Conical) => half_triangle(props),
        Some(Boyshp::Can) => rhomboid(props),
        Some(Boyshp::Spherical) => circle(props),
        Some(Boyshp::Pillar) | Some(Boyshp::Spar) => stake(props),
        Some(Boyshp::Barrel) | Some(Boyshp::SuperBuoy) | Some(Boyshp::IceBuoy) => circle(props),
        None => "BOYDEF03",
    }
}

pub fn style_feature(props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
    let mut style = FeatureStyle::default();
    let sym = match catlam(props) {
        Some(Catlam::PortHand) => rhomboid(props),
        Some(Catlam::StarboardHand) => half_triangle(props),
        Some(Catlam::PreferredToStarboard) => rhomboid(props),
        Some(Catlam::PreferredToPort) => half_triangle(props),
        None => by_boyshp(props),
    };
    style.symbol = Some(sym.to_string());
    style
}
