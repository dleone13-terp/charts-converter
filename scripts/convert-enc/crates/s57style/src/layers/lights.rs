use crate::color::ColorLibrary;
use crate::config::{StyleConfig, Theme};
use crate::feature::{PropMap, PropMapExt};
use crate::style::FeatureStyle;
use super::attr::colour::{first_colour, Colour};

pub fn style_feature(props: &PropMap, config: &StyleConfig) -> FeatureStyle {
    let mut style = FeatureStyle::default();

    let light_color = match first_colour(props) {
        Some(Colour::Red) => {
            style.symbol = Some("LIGHTS11".to_string());
            style.line_color = Some("LITRD".to_string());
            "LITRD"
        }
        Some(Colour::Green) => {
            style.symbol = Some("LIGHTS12".to_string());
            style.line_color = Some("LITGN".to_string());
            "LITGN"
        }
        Some(Colour::Yellow) | Some(Colour::White) | Some(Colour::Amber) | Some(Colour::Orange) => {
            style.symbol = Some("LIGHTS13".to_string());
            style.line_color = Some("LITYW".to_string());
            "LITYW"
        }
        _ => {
            style.symbol = Some("LITDEF11".to_string());
            style.line_color = Some("LITYW".to_string());
            "LITYW"
        }
    };

    let lib = ColorLibrary::instance();
    let day_color = lib.resolve(light_color, Theme::Day).unwrap_or("#FFFFFF");
    let dusk_color = lib.resolve(light_color, Theme::Dusk).unwrap_or("#FFFFFF");
    let night_color = lib.resolve(light_color, Theme::Night).unwrap_or("#FFFFFF");
    let day_line = lib.resolve("CHBLK", Theme::Day).unwrap_or("#000000");
    let dusk_line = lib.resolve("CHBLK", Theme::Dusk).unwrap_or("#000000");
    let night_line = lib.resolve("CHBLK", Theme::Night).unwrap_or("#000000");

    let radius: u32 = match light_color {
        "LITRD" => 68,
        "LITGN" => 74,
        _ => 80,
    };

    let sectr1 = props.get_float("SECTR1");
    let sectr2 = props.get_float("SECTR2");
    let valnmr = props.get_float("VALNMR");
    let major_light = valnmr.map_or(false, |v| v >= 10.0);

    if major_light || (sectr1.is_some() && sectr2.is_some()) {
        let s1 = sectr1.unwrap_or(0.0);
        let s2 = sectr2.unwrap_or(0.0);
        let si = format!(
            "sector_{s1}_{s2}_{day_color}_{dusk_color}_{night_color}_{day_line}_{dusk_line}_{night_line}_{radius}"
        );
        style.extra.insert("SI".to_string(), serde_json::Value::String(si));
    }

    let _ = config; // config not needed beyond sector radius, which is hardcoded per S-57
    style
}
