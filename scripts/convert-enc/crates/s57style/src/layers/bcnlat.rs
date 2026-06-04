use crate::config::StyleConfig;
use crate::feature::PropMap;
use crate::style::FeatureStyle;
use super::attr::bcnshp::{bcnshp, Bcnshp};
use super::attr::colour::{first_colour, Colour};

pub fn style_feature(props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
    let mut style = FeatureStyle::default();
    let sym = match bcnshp(props) {
        Some(Bcnshp::StakePolePerchPost) | Some(Bcnshp::Whity) => {
            match first_colour(props) {
                Some(Colour::Red) => "BCNLAT21",
                Some(Colour::Green) => "BCNLAT22",
                Some(Colour::Black) => "BCNSAW21",
                _ => "BCNSPP21",
            }
        }
        Some(Bcnshp::BeaconTower) | Some(Bcnshp::LatticeBeacon) | Some(Bcnshp::PileBeacon) => {
            match first_colour(props) {
                Some(Colour::Red) => "BCNLAT15",
                Some(Colour::Green) => "BCNLAT16",
                Some(Colour::Black) => "BCNSAW13",
                _ => "BCNSPP13",
            }
        }
        Some(Bcnshp::Cairn) => "CAIRNS11",
        Some(Bcnshp::BuoyantBeacon) => "BCNLAT21",
        None => "BCNDEF13",
    };
    style.symbol = Some(sym.to_string());
    style
}
