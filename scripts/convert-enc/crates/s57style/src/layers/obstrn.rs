use crate::config::StyleConfig;
use crate::feature::{PropMap, PropMapExt};
use crate::style::FeatureStyle;
use super::attr::catobs::{catobs, Catobs};
use super::attr::depth_color::DepthColor;
use super::attr::quasou::{quasou, Quasou};
use super::attr::watlev::{watlev, Watlev};

fn add_sounding_conversions(style: &mut FeatureStyle, meters: f64) {
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

pub fn style_feature(props: &PropMap, config: &StyleConfig) -> FeatureStyle {
    let mut style = FeatureStyle::default();

    let valsou = props.get_float("VALSOU");
    let category = catobs(props);
    let water_level = watlev(props);
    let quasou_list = quasou(props);

    // Determine if depth value is usable
    let usable_depth = quasou_list.iter().any(|q| matches!(
        q,
        Quasou::DepthKnown
            | Quasou::NoBottomFound
            | Quasou::LeastDepthKnown
            | Quasou::LeastDepthUnknown
            | Quasou::MaintainedDepth
    ));

    let mut sy_set = false;
    let mut show_depth = usable_depth;
    let mut fill_depth_color = true;

    match category {
        Some(Catobs::SnagStump)
        | Some(Catobs::Wellhead)
        | Some(Catobs::Diffuser)
        | Some(Catobs::Crib) => {
            if !usable_depth {
                style.symbol = Some("ISODGR01".to_string());
                sy_set = true;
                show_depth = false;
            }
        }
        Some(Catobs::FishHaven) => {
            style.symbol = Some("FSHHAV01".to_string());
            sy_set = true;
            show_depth = false;
        }
        Some(Catobs::FoulArea) | Some(Catobs::FoulGround) => {
            style.area_pattern = Some("FOULAR01P".to_string());
        }
        Some(Catobs::GroundTackle) => {
            style.symbol = Some("ACHARE02".to_string());
            sy_set = true;
            show_depth = false;
        }
        Some(Catobs::IceBoom) | Some(Catobs::Boom) => {
            style.symbol = Some("FLTHAZ02".to_string());
        }
        None => {}
    }

    // Compute depth color based on water level and depth
    let depth_color = {
        match water_level {
            Some(Watlev::AlwaysUnderwater) => {
                if let Some(m) = valsou.filter(|_| usable_depth) {
                    if m <= config.shallow_depth {
                        DepthColor::VeryShallow
                    } else if m <= config.safety_depth {
                        DepthColor::SafetyDepth
                    } else if m <= config.deep_depth {
                        DepthColor::MediumDepth
                    } else {
                        DepthColor::DeepWater
                    }
                } else {
                    DepthColor::VeryShallow
                }
            }
            Some(Watlev::PartlySubmerged)
            | Some(Watlev::AlwaysDry)
            | Some(Watlev::CoversAndUncovers)
            | Some(Watlev::Awash)
            | Some(Watlev::SubjectToInundation)
            | Some(Watlev::Floating)
            | None => DepthColor::CoversUncovers,
        }
    };

    let mut check_accuracy = true;
    if !sy_set {
        match water_level {
            Some(Watlev::CoversAndUncovers) => {
                check_accuracy = false;
                style.exclude_area_point_symbol = true;
                style.symbol = Some("OBSTRN03".to_string());
            }
            Some(Watlev::AlwaysDry) => {
                check_accuracy = false;
                style.exclude_area_point_symbol = true;
                style.symbol = Some("OBSTRN11".to_string());
            }
            Some(Watlev::AlwaysUnderwater) => {
                match depth_color {
                    DepthColor::DeepWater | DepthColor::MediumDepth => {
                        style.exclude_area_point_symbol = true;
                        style.symbol = Some(
                            if show_depth { "DANGER02" } else { "OBSTRN02" }.to_string()
                        );
                    }
                    DepthColor::SafetyDepth | DepthColor::VeryShallow => {
                        style.exclude_area_point_symbol = true;
                        style.symbol = Some(
                            if show_depth { "DANGER01" } else { "OBSTRN01" }.to_string()
                        );
                    }
                    DepthColor::CoversUncovers => {
                        style.exclude_area_point_symbol = true;
                        style.symbol = Some(
                            if show_depth { "DANGER03" } else { "OBSTRN03" }.to_string()
                        );
                    }
                }
            }
            Some(Watlev::Floating) => {
                show_depth = false;
                fill_depth_color = false;
                style.symbol = Some("FLTHAZ02".to_string());
            }
            Some(Watlev::PartlySubmerged)
            | Some(Watlev::Awash)
            | Some(Watlev::SubjectToInundation)
            | None => {
                check_accuracy = false;
                style.symbol = Some("ISODGR01".to_string());
            }
        }
    }

    if check_accuracy
        && quasou_list.iter().any(|q| matches!(q, Quasou::Doubtful | Quasou::Unreliable))
    {
        show_depth = false;
        style.symbol = Some("LOWACC01".to_string());
    }

    if fill_depth_color {
        style.area_color = Some(depth_color.color_name().to_string());
    }

    if show_depth {
        if let Some(meters) = valsou {
            add_sounding_conversions(&mut style, meters);
        }
    }

    style
}
