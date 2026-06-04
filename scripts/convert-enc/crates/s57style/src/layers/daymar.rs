use crate::config::StyleConfig;
use crate::feature::PropMap;
use crate::style::FeatureStyle;
use super::attr::topshp::{topshp, Topshp};

pub fn style_feature(props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
    let mut style = FeatureStyle::default();

    let sym = match topshp(props) {
        Some(Topshp::Square) | Some(Topshp::RectangleHorizontal) | Some(Topshp::RectangleVertical) => {
            "DAYSQR01"
        }
        Some(Topshp::TrianglePointUp) => "DAYTRI01",
        Some(Topshp::TrianglePointDown) => "DAYTRI05",
        _ => "DAYSQR01",
    };

    style.symbol = Some(sym.to_string());
    style
}
