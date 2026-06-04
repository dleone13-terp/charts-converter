use crate::config::StyleConfig;
use crate::feature::PropMap;
use crate::style::FeatureStyle;

pub fn style_feature(_props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
    FeatureStyle {
        symbol: Some("BOYISD21".to_string()),
        ..Default::default()
    }
}
