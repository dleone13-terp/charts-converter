use crate::config::StyleConfig;
use crate::feature::{PropMap, PropMapExt};
use crate::style::FeatureStyle;

pub fn style_feature(props: &PropMap, config: &StyleConfig) -> FeatureStyle {
    let mut style = FeatureStyle::default();

    // DRVAL1 is the shallower bound, DRVAL2 is the deeper bound
    if let (Some(drval1), Some(drval2)) = (props.get_float("DRVAL1"), props.get_float("DRVAL2")) {
        let ac = if drval1 < 0.0 && drval2 <= 0.0 {
            "DEPIT"
        } else if drval1 <= config.shallow_depth {
            "DEPVS"
        } else if drval1 <= config.safety_depth {
            "DEPMS"
        } else if drval1 <= config.deep_depth {
            "DEPMD"
        } else {
            "DEPDW"
        };
        style.area_color = Some(ac.to_string());
    }

    style.line_color = Some("CHGRD".to_string());
    style
}
