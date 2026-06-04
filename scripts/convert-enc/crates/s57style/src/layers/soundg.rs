use crate::config::StyleConfig;
use crate::feature::{PropMap, PropMapExt};
use crate::style::FeatureStyle;

pub fn style_feature(props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
    let mut style = FeatureStyle::default();

    if let Some(meters) = props.get_float("METERS") {
        let fathoms = meters / 1.8288;
        let fathoms_whole = fathoms.floor() as i64;
        let fathoms_ft = ((fathoms - fathoms_whole as f64) * 12.0).round() as i64;
        let feet = meters * 3.28084;
        let meters_w = meters.floor() as i64;
        let meters_t = ((meters - meters_w as f64) * 10.0).round() as i64;

        style.extra.insert("METERS".to_string(), serde_json::json!(meters));
        style.extra.insert("FATHOMS".to_string(), serde_json::json!(fathoms_whole));
        style.extra.insert("FATHOMS_FT".to_string(), serde_json::json!(fathoms_ft));
        style.extra.insert("FEET".to_string(), serde_json::json!(feet.floor() as i64));
        style.extra.insert("METERS_W".to_string(), serde_json::json!(meters_w));
        style.extra.insert("METERS_T".to_string(), serde_json::json!(meters_t));
    }

    style
}
