use std::collections::HashMap;
use once_cell::sync::Lazy;
use crate::config::Theme;

static COLOR_JSON: &str = include_str!("../../../resources/colors.json");

/// A library of S-57 color names mapped to hex values for each display theme.
pub struct ColorLibrary {
    day: HashMap<String, String>,
    dusk: HashMap<String, String>,
    night: HashMap<String, String>,
}

impl ColorLibrary {
    fn load() -> Self {
        let v: serde_json::Value =
            serde_json::from_str(COLOR_JSON).expect("colors.json is valid JSON");
        let lib = &v["library"];

        let parse_theme = |key: &str| -> HashMap<String, String> {
            lib[key]
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                })
                .unwrap_or_default()
        };

        ColorLibrary {
            day: parse_theme("DAY"),
            dusk: parse_theme("DUSK"),
            night: parse_theme("NIGHT"),
        }
    }

    /// Get the global singleton instance.
    pub fn instance() -> &'static Self {
        static INSTANCE: Lazy<ColorLibrary> = Lazy::new(ColorLibrary::load);
        &INSTANCE
    }

    /// Resolve an S-57 color name to a hex string for the given theme.
    pub fn resolve(&self, s57_color: &str, theme: Theme) -> Option<&str> {
        let map = match theme {
            Theme::Day => &self.day,
            Theme::Dusk => &self.dusk,
            Theme::Night => &self.night,
        };
        map.get(s57_color).map(|s| s.as_str())
    }
}
