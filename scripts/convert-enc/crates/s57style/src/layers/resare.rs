use crate::config::StyleConfig;
use crate::feature::PropMap;
use crate::style::FeatureStyle;
use super::attr::restrn::{restrn, Restrn};

pub fn style_feature(props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
    let mut style = FeatureStyle::default();

    let restrictions = restrn(props);

    if restrictions.iter().any(|r| matches!(r, Restrn::EntryRestricted | Restrn::EntryProhibited)) {
        style.symbol = Some("ENTRES51".to_string());
    }

    if restrictions.iter().any(|r| matches!(r, Restrn::AnchoringProhibited | Restrn::AnchoringRestricted)) {
        style.symbol = Some("ACHRES51".to_string());
    }

    style.line_color = Some("CHMGD".to_string());
    style
}
