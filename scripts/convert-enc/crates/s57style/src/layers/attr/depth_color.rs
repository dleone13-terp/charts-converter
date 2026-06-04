use crate::config::StyleConfig;
use super::watlev::Watlev;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DepthColor {
    DeepWater,
    MediumDepth,
    SafetyDepth,
    VeryShallow,
    CoversUncovers,
}

impl DepthColor {
    /// Returns the S-57 color name for this depth color.
    pub fn color_name(&self) -> &'static str {
        match self {
            DepthColor::DeepWater => "DEPDW",
            DepthColor::MediumDepth => "DEPMD",
            DepthColor::SafetyDepth => "DEPMS",
            DepthColor::VeryShallow => "DEPVS",
            DepthColor::CoversUncovers => "DEPIT",
        }
    }

    /// Compute depth color from a depth in meters using style config thresholds.
    pub fn from_depth(depth: f64, config: &StyleConfig) -> Self {
        if depth < 0.0 {
            DepthColor::CoversUncovers
        } else if depth <= config.shallow_depth {
            DepthColor::VeryShallow
        } else if depth <= config.safety_depth {
            DepthColor::SafetyDepth
        } else if depth <= config.deep_depth {
            DepthColor::MediumDepth
        } else {
            DepthColor::DeepWater
        }
    }

    /// Compute depth color considering water level effect and optional sounding depth.
    pub fn from_watlev_depth(
        watlev: Option<Watlev>,
        depth: Option<f64>,
        config: &StyleConfig,
    ) -> Self {
        match watlev {
            Some(Watlev::AlwaysUnderwater) => {
                if let Some(d) = depth {
                    if d <= config.shallow_depth {
                        DepthColor::VeryShallow
                    } else if d <= config.safety_depth {
                        DepthColor::SafetyDepth
                    } else if d <= config.deep_depth {
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
    }
}
