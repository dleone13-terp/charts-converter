/// Display theme (day/dusk/night)
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Theme {
    Day,
    Dusk,
    Night,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Day
    }
}

/// Depth unit for display conversions
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DepthUnit {
    Meters,
    Fathoms,
    Feet,
}

impl Default for DepthUnit {
    fn default() -> Self {
        DepthUnit::Meters
    }
}

/// Configuration for the style engine.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StyleConfig {
    /// Shallow contour depth in meters (default 3.0)
    pub shallow_depth: f64,
    /// Safety contour depth in meters (default 6.0)
    pub safety_depth: f64,
    /// Deep contour depth in meters (default 9.0)
    pub deep_depth: f64,
    /// Display theme
    pub theme: Theme,
    /// Depth unit
    pub depth_unit: DepthUnit,
}

impl Default for StyleConfig {
    fn default() -> Self {
        StyleConfig {
            shallow_depth: 3.0,
            safety_depth: 6.0,
            deep_depth: 9.0,
            theme: Theme::Day,
            depth_unit: DepthUnit::Meters,
        }
    }
}
