//! S-57 nautical chart styling engine.
//!
//! Takes GeoJSON feature properties, applies S-57 conditional styling rules,
//! and returns style information (symbols, colors, patterns).

pub mod color;
pub mod config;
pub mod engine;
pub mod feature;
pub mod layers;
pub mod mapbox;
pub mod sprite;
pub mod style;

pub use color::ColorLibrary;
pub use config::{DepthUnit, StyleConfig, Theme};
pub use engine::StyleEngine;
pub use feature::{PropMap, PropMapExt};
pub use style::{FeatureStyle, ResolvedFeatureStyle};
