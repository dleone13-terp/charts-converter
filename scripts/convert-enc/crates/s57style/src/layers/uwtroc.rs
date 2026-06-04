use crate::config::StyleConfig;
use crate::feature::PropMap;
use crate::style::FeatureStyle;
use super::attr::watlev::{watlev, Watlev};

pub fn style_feature(props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
    let mut style = FeatureStyle::default();

    let sym = match watlev(props) {
        Some(Watlev::AlwaysUnderwater) => "UWTROC04",
        Some(Watlev::Awash)
        | Some(Watlev::PartlySubmerged)
        | Some(Watlev::CoversAndUncovers)
        | Some(Watlev::SubjectToInundation) => "UWTROC03",
        Some(Watlev::AlwaysDry) | Some(Watlev::Floating) | None => "ISODGR01",
    };

    style.symbol = Some(sym.to_string());
    style
}
