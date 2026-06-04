use crate::config::StyleConfig;
use crate::feature::PropMap;
use crate::style::FeatureStyle;
use super::attr::catcam::{catcam, Catcam};

pub fn style_feature(props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
    let mut style = FeatureStyle::default();
    style.symbol = Some(match catcam(props) {
        Some(Catcam::NorthCardinalMark) => "BCNCAR01",
        Some(Catcam::EastCardinalMark) => "BCNCAR02",
        Some(Catcam::SouthCardinalMark) => "BCNCAR03",
        Some(Catcam::WestCardinalMark) => "BCNCAR04",
        None => "BCNDEF13",
    }.to_string());
    style
}
