use crate::config::StyleConfig;
use crate::feature::{PropMap, PropMapExt};
use crate::style::FeatureStyle;
use super::attr::catwrk::{catwrk, Catwrk};
use super::attr::watlev::{watlev, Watlev};

pub fn style_feature(props: &PropMap, config: &StyleConfig) -> FeatureStyle {
    let mut style = FeatureStyle::default();

    let sym = match catwrk(props) {
        Some(Catwrk::Dangerous) => "WRECKS05",
        Some(Catwrk::ShowingMast) | Some(Catwrk::ShowingHull) => "WRECKS01",
        Some(Catwrk::NonDangerous) | Some(Catwrk::Distributed) | None => {
            let valsou = props.get_float("VALSOU");
            let under_safety = valsou.map(|v| v > config.safety_depth);

            if watlev(props) == Some(Watlev::AlwaysUnderwater)
                && (under_safety.is_none() || under_safety == Some(true))
            {
                "WRECKS04"
            } else if under_safety == Some(false) {
                "DANGER01"
            } else {
                "ISODGR01"
            }
        }
    };

    style.symbol = Some(sym.to_string());
    style
}
