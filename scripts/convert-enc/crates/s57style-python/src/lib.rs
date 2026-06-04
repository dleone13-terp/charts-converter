use pyo3::prelude::*;
use s57style::{DepthUnit, PropMap, StyleConfig, StyleEngine, Theme};

/// Python-exposed style engine.
#[pyclass(name = "StyleEngine")]
struct PyStyleEngine {
    inner: StyleEngine,
}

#[pymethods]
impl PyStyleEngine {
    /// Create a new engine.
    ///
    /// Args:
    ///     shallow_depth: Shallow contour depth in meters (default 3.0)
    ///     safety_depth: Safety contour depth in meters (default 6.0)
    ///     deep_depth: Deep contour depth in meters (default 9.0)
    ///     theme: "Day", "Dusk", or "Night"
    ///     depth_unit: "Meters", "Fathoms", or "Feet"
    #[new]
    #[pyo3(signature = (shallow_depth=3.0, safety_depth=6.0, deep_depth=9.0, theme="Day", depth_unit="Meters"))]
    fn new(
        shallow_depth: f64,
        safety_depth: f64,
        deep_depth: f64,
        theme: &str,
        depth_unit: &str,
    ) -> PyResult<Self> {
        let theme = parse_theme(theme)?;
        let depth_unit = parse_depth_unit(depth_unit)?;
        Ok(PyStyleEngine {
            inner: StyleEngine::new(StyleConfig {
                shallow_depth,
                safety_depth,
                deep_depth,
                theme,
                depth_unit,
            }),
        })
    }

    /// Style a feature. Returns FeatureStyle as a JSON string.
    ///
    /// Args:
    ///     layer: S-57 layer name (e.g. "DEPARE", "LIGHTS")
    ///     props_json: Feature properties as a JSON string
    fn style_feature(&self, layer: &str, props_json: &str) -> PyResult<String> {
        let props: PropMap = serde_json::from_str(props_json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid JSON: {e}")))?;
        let style = self.inner.style_feature(layer, &props);
        serde_json::to_string(&style)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Serialize error: {e}")))
    }

    /// Style a feature with colors resolved to hex. Returns ResolvedFeatureStyle as JSON string.
    fn style_feature_resolved(&self, layer: &str, props_json: &str) -> PyResult<String> {
        let props: PropMap = serde_json::from_str(props_json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid JSON: {e}")))?;
        let style = self.inner.style_feature_resolved(layer, &props);
        serde_json::to_string(&style)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Serialize error: {e}")))
    }

    /// Resolve an S-57 color name to a hex string.
    fn resolve_color(&self, s57_name: &str) -> Option<String> {
        self.inner.resolve_color(s57_name).map(|s| s.to_string())
    }

    /// Generate a Mapbox GL style JSON document.
    #[pyo3(signature = (source_url, sprite_url=None, glyphs_url=None))]
    fn mapbox_style_json(
        &self,
        source_url: &str,
        sprite_url: Option<&str>,
        glyphs_url: Option<&str>,
    ) -> String {
        self.inner.mapbox_style_json(source_url, sprite_url, glyphs_url)
    }

    /// Style a full GeoJSON Feature dict (as JSON string).
    ///
    /// Expects `{"type":"Feature","geometry":{...},"properties":{"layer":"DEPARE",...}}`.
    /// Extracts layer name from properties["layer"], computes style, and merges
    /// style properties back into the feature's properties.
    ///
    /// Returns the modified GeoJSON Feature as a JSON string.
    fn style_geojson_feature(&self, geojson_json: &str) -> PyResult<String> {
        let mut feature: serde_json::Value = serde_json::from_str(geojson_json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid GeoJSON: {e}")))?;

        let layer = feature
            .get("properties")
            .and_then(|p| p.get("layer"))
            .and_then(|l| l.as_str())
            .unwrap_or("")
            .to_string();

        let props: PropMap = feature
            .get("properties")
            .and_then(|p| p.as_object())
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default();

        let style = self.inner.style_feature(&layer, &props);

        // Merge style into properties
        if let Some(properties) = feature.get_mut("properties").and_then(|p| p.as_object_mut()) {
            if let Some(sym) = style.symbol {
                properties.insert("SY".to_string(), serde_json::json!(sym));
            }
            if let Some(ac) = style.area_color {
                properties.insert("AC".to_string(), serde_json::json!(ac));
            }
            if let Some(lc) = style.line_color {
                properties.insert("LC".to_string(), serde_json::json!(lc));
            }
            if let Some(ap) = style.area_pattern {
                properties.insert("AP".to_string(), serde_json::json!(ap));
            }
            if let Some(lp) = style.line_pattern {
                properties.insert("LP".to_string(), serde_json::json!(lp));
            }
            if style.exclude_area_point_symbol {
                properties.insert("EA".to_string(), serde_json::json!(true));
            }
            for (k, v) in style.extra {
                properties.insert(k, v);
            }
        }

        serde_json::to_string(&feature)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Serialize error: {e}")))
    }
}

fn parse_theme(s: &str) -> PyResult<Theme> {
    match s {
        "Day" | "day" => Ok(Theme::Day),
        "Dusk" | "dusk" => Ok(Theme::Dusk),
        "Night" | "night" => Ok(Theme::Night),
        other => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Unknown theme '{other}'. Use 'Day', 'Dusk', or 'Night'."
        ))),
    }
}

fn parse_depth_unit(s: &str) -> PyResult<DepthUnit> {
    match s {
        "Meters" | "meters" => Ok(DepthUnit::Meters),
        "Fathoms" | "fathoms" => Ok(DepthUnit::Fathoms),
        "Feet" | "feet" => Ok(DepthUnit::Feet),
        other => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Unknown depth unit '{other}'. Use 'Meters', 'Fathoms', or 'Feet'."
        ))),
    }
}

#[pymodule]
#[pyo3(name = "s57style")]
fn _s57style_module(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyStyleEngine>()?;
    Ok(())
}
