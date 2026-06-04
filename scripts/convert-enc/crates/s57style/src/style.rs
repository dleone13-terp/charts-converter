use std::collections::HashMap;
use serde_json::Value;

/// The raw S-57 style for a feature. Color fields contain S-57 color names (e.g. "DEPVS").
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FeatureStyle {
    /// Point symbol name (SY)
    pub symbol: Option<String>,
    /// Area fill color — S57 color name like "DEPVS" (AC)
    pub area_color: Option<String>,
    /// Line color — S57 color name (LC)
    pub line_color: Option<String>,
    /// Area fill pattern sprite name (AP)
    pub area_pattern: Option<String>,
    /// Line pattern sprite name (LP)
    pub line_pattern: Option<String>,
    /// Exclude area point symbol (EA)
    pub exclude_area_point_symbol: bool,
    /// Additional computed properties (e.g. depth conversions, sector info)
    pub extra: HashMap<String, Value>,
}

/// A resolved feature style where color fields contain hex codes (e.g. "#61B7FF").
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ResolvedFeatureStyle {
    /// Point symbol name
    pub symbol: Option<String>,
    /// Area fill color as hex (e.g. "#61B7FF")
    pub area_color: Option<String>,
    /// Line color as hex
    pub line_color: Option<String>,
    /// Area fill pattern sprite name
    pub area_pattern: Option<String>,
    /// Line pattern sprite name
    pub line_pattern: Option<String>,
    /// Exclude area point symbol
    pub exclude_area_point_symbol: bool,
    /// Additional computed properties
    pub extra: HashMap<String, Value>,
}
