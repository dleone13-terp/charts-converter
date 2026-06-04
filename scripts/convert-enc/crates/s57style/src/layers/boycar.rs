use crate::config::StyleConfig;
use crate::feature::PropMap;
use crate::style::FeatureStyle;
use super::attr::catcam::{catcam, Catcam};

pub fn style_feature(props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
    let mut style = FeatureStyle::default();
    style.symbol = Some(match catcam(props) {
        Some(Catcam::NorthCardinalMark) => "BOYCAR01",
        Some(Catcam::EastCardinalMark) => "BOYCAR02",
        Some(Catcam::SouthCardinalMark) => "BOYCAR03",
        Some(Catcam::WestCardinalMark) => "BOYCAR04",
        None => "BOYDEF03",
    }.to_string());
    style
}
