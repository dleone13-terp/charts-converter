use crate::config::StyleConfig;
use crate::feature::PropMap;
use crate::style::FeatureStyle;

pub fn style_feature(_props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
    FeatureStyle {
        symbol: Some("LNDARE01".to_string()),
        area_color: Some("LANDA".to_string()),
        ..Default::default()
    }
}
