use crate::color::ColorLibrary;
use crate::config::StyleConfig;
use crate::feature::PropMap;
use crate::layers::registry::{build_registry, StyleFn};
use crate::mapbox::build_mapbox_style_json;
use crate::style::{FeatureStyle, ResolvedFeatureStyle};
use once_cell::sync::Lazy;
use std::collections::HashMap;

static REGISTRY: Lazy<HashMap<&'static str, StyleFn>> = Lazy::new(build_registry);

/// The main styling engine. Create one per configuration.
pub struct StyleEngine {
    config: StyleConfig,
    color_lib: &'static ColorLibrary,
}

impl StyleEngine {
    /// Create a new `StyleEngine` with the given configuration.
    pub fn new(config: StyleConfig) -> Self {
        StyleEngine {
            config,
            color_lib: ColorLibrary::instance(),
        }
    }

    /// Apply S-57 conditional styling to a feature, returning S-57 color names.
    pub fn style_feature(&self, layer: &str, props: &PropMap) -> FeatureStyle {
        if let Some(f) = REGISTRY.get(layer) {
            f(props, &self.config)
        } else {
            FeatureStyle::default()
        }
    }

    /// Apply S-57 conditional styling and resolve all color names to hex values.
    pub fn style_feature_resolved(&self, layer: &str, props: &PropMap) -> ResolvedFeatureStyle {
        let style = self.style_feature(layer, props);
        let theme = self.config.theme;
        let lib = self.color_lib;

        ResolvedFeatureStyle {
            symbol: style.symbol,
            area_color: style
                .area_color
                .as_deref()
                .and_then(|c| lib.resolve(c, theme))
                .map(|s| s.to_string()),
            line_color: style
                .line_color
                .as_deref()
                .and_then(|c| lib.resolve(c, theme))
                .map(|s| s.to_string()),
            area_pattern: style.area_pattern,
            line_pattern: style.line_pattern,
            exclude_area_point_symbol: style.exclude_area_point_symbol,
            extra: style.extra,
        }
    }

    /// Resolve an S-57 color name to a hex string for the current theme.
    pub fn resolve_color(&self, s57_name: &str) -> Option<&str> {
        self.color_lib.resolve(s57_name, self.config.theme)
    }

    /// Generate a Mapbox GL style JSON document.
    pub fn mapbox_style_json(
        &self,
        source_url: &str,
        sprite_url: Option<&str>,
        glyphs_url: Option<&str>,
    ) -> String {
        build_mapbox_style_json(source_url, sprite_url, glyphs_url, self.config.theme, self.config.depth_unit)
    }
}
