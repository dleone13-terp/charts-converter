use serde_json::{json, Value};
use crate::color::ColorLibrary;
use crate::config::{DepthUnit, Theme};

/// Build a Mapbox GL `case` expression that maps an S-57 color name property
/// to a hex color, with a transparent fallback.
pub fn color_case_expr(
    prop_key: &str,
    color_names: &[&str],
    theme: Theme,
    lib: &ColorLibrary,
) -> Value {
    let mut parts: Vec<Value> = vec![json!("case")];
    for name in color_names {
        if let Some(hex) = lib.resolve(name, theme) {
            parts.push(json!(["==", ["get", prop_key], name]));
            parts.push(json!(hex));
        }
    }
    parts.push(json!("rgba(0,0,0,0)"));
    Value::Array(parts)
}

/// All S-57 area color names
const AREA_COLORS: &[&str] = &[
    "DEPDW", "DEPMD", "DEPMS", "DEPVS", "DEPIT",
    "LANDA", "CHBRN", "TRFCF", "NODTA", "DEPDW",
    "CHWHT", "CHBLK", "CHMGF",
];

/// All S-57 line color names
const LINE_COLORS: &[&str] = &[
    "CHGRD", "CSTLN", "CHBLK", "CHMGD", "TRFCD",
    "CHBRN", "CHRED", "CHYLW", "RADHI", "DEPDW",
    "CHGRN", "LANDF",
];

/// Build the full Mapbox GL style JSON document.
pub fn build_mapbox_style_json(
    source_url: &str,
    sprite_url: Option<&str>,
    glyphs_url: Option<&str>,
    theme: Theme,
    depth_unit: DepthUnit,
) -> String {
    let lib = ColorLibrary::instance();

    let sprite = sprite_url.unwrap_or("https://example.com/sprites/sprite");
    let glyphs = glyphs_url.unwrap_or("https://example.com/fonts/{fontstack}/{range}.pbf");

    let bg_color = lib.resolve("DEPDW", theme).unwrap_or("#C9EDFF");

    let mut layers: Vec<Value> = vec![];

    // Background
    layers.push(json!({
        "id": "background",
        "type": "background",
        "paint": { "background-color": bg_color }
    }));

    // Helper closures for building layer objects
    let area_color_expr = color_case_expr("AC", AREA_COLORS, theme, lib);
    let line_color_expr = color_case_expr("LC", LINE_COLORS, theme, lib);

    // --- Fill layers ---
    for layer_name in POLYGON_LAYERS {
        layers.push(json!({
            "id": format!("{}_fill", layer_name),
            "type": "fill",
            "source": "senc",
            "source-layer": layer_name,
            "filter": ["==", "$type", "Polygon"],
            "paint": {
                "fill-color": area_color_expr.clone(),
                "fill-opacity": 0.8
            }
        }));
    }

    // --- Line layers ---
    for layer_name in LINE_LAYERS {
        layers.push(json!({
            "id": format!("{}_line", layer_name),
            "type": "line",
            "source": "senc",
            "source-layer": layer_name,
            "filter": ["!=", "$type", "Point"],
            "paint": {
                "line-color": line_color_expr.clone(),
                "line-width": 1.0
            }
        }));
    }

    // --- Symbol layers (point icons) ---
    for layer_name in SYMBOL_LAYERS {
        layers.push(json!({
            "id": format!("{}_symbol", layer_name),
            "type": "symbol",
            "source": "senc",
            "source-layer": layer_name,
            "filter": ["==", "$type", "Point"],
            "layout": {
                "icon-image": ["get", "SY"],
                "icon-anchor": "bottom",
                "icon-allow-overlap": true,
                "icon-size": 1.0
            }
        }));
    }

    // LIGHTS sector layer
    layers.push(json!({
        "id": "LIGHTS_sector",
        "type": "symbol",
        "source": "senc",
        "source-layer": "LIGHTS",
        "filter": ["all", ["==", "$type", "Point"], ["has", "SI"]],
        "layout": {
            "icon-image": ["get", "SI"],
            "icon-anchor": "center",
            "icon-allow-overlap": true,
            "icon-size": 1.0
        }
    }));

    // LIGHTS regular (no sector)
    layers.push(json!({
        "id": "LIGHTS_symbol",
        "type": "symbol",
        "source": "senc",
        "source-layer": "LIGHTS",
        "filter": ["all", ["==", "$type", "Point"], ["!has", "SI"]],
        "layout": {
            "icon-image": ["get", "SY"],
            "icon-anchor": "bottom",
            "icon-allow-overlap": true,
            "icon-size": 1.0,
            "icon-rotate": 135.0
        }
    }));

    // SOUNDG depth text layer — label format depends on depth unit
    let sndg2_color = lib.resolve("SNDG2", theme).unwrap_or("#000000");
    // ogr2ogr ADD_SOUNDG_DEPTH=YES stamps a float "DEPTH" property (meters) on each sounding.
    let soundg_text_field: Value = match depth_unit {
        DepthUnit::Meters => json!(["to-string", ["get", "DEPTH"]]),
        // 1 m = 0.546807 fm
        DepthUnit::Fathoms => json!([
            "concat",
            ["to-string", ["floor", ["*", ["get", "DEPTH"], 0.546_807]]],
            "f"
        ]),
        // 1 m = 3.28084 ft
        DepthUnit::Feet => json!([
            "concat",
            ["to-string", ["floor", ["*", ["get", "DEPTH"], 3.280_84]]],
            "'"
        ]),
    };
    layers.push(json!({
        "id": "SOUNDG_text",
        "type": "symbol",
        "source": "senc",
        "source-layer": "SOUNDG",
        "filter": ["==", "$type", "Point"],
        "layout": {
            "text-field": soundg_text_field,
            "text-font": ["Roboto Bold"],
            "text-size": 10,
            "text-anchor": "center",
            "text-allow-overlap": false
        },
        "paint": {
            "text-color": sndg2_color
        }
    }));

    let style = json!({
        "version": 8,
        "name": "openenc",
        "sprite": sprite,
        "glyphs": glyphs,
        "sources": {
            "senc": {
                "type": "vector",
                "url": source_url
            }
        },
        "layers": layers
    });

    serde_json::to_string_pretty(&style).unwrap_or_default()
}

/// Layers that need fill rendering
const POLYGON_LAYERS: &[&str] = &[
    "DEPARE", "LNDARE", "SLOGRD", "LAKARE", "BUISGL", "BUAARE",
    "OBSTRN", "LNDRGN", "DRGARE", "ICEARE", "CAUSWY", "CANALS",
    "RIVERS", "AIRARE", "RUNWAY", "DYKCON", "LOKBSN", "LOGPON",
    "DOCARE", "FLODOC", "WEDKLP", "FAIRWY", "TSSLPT", "TSSCRS",
    "TSSRON", "DWRTPT", "TWRTPT", "TSEZNE", "RCTLPT", "RECTRC",
    "PONTON", "BRIDGE", "DAMCON", "HULKES", "PRDARE", "RESARE",
    "CTNARE", "ICNARE", "ACHARE", "CBLARE", "SPLARE", "FORSTC",
];

/// Layers that need line rendering
const LINE_LAYERS: &[&str] = &[
    "DEPARE", "COALNE", "LAKSHR", "SLCONS", "CBLSUB", "CBLOHD",
    "RESARE", "FAIRWY", "TSSBND", "TSELNE", "TWRTPT", "DWRTCL",
    "MIPARE", "CBLARE", "NAVLNE", "RADLNE", "RADRNG", "RCRTCL",
    "UNSARE", "DMPGRD", "FNCLNE", "FERYRT", "FSHGRD", "SUBTLN",
    "SWPARE", "TCTLPT", "PIPARE", "OILBAR", "PRCARE", "SPLARE",
    "TUNNEL", "ISTZNE",
];

/// Layers that need symbol (icon) rendering
const SYMBOL_LAYERS: &[&str] = &[
    "BCNCAR", "BCNISD", "BCNLAT", "BCNSAW", "BCNSPP",
    "BOYCAR", "BOYISD", "BOYLAT", "BOYSAW", "BOYSPP", "BOYINB",
    "TOPMAR", "DAYMAR", "OBSTRN", "WRECKS", "UWTROC",
    "LNDMRK", "HULKES", "MORFAC", "ACHBRT", "ACHPNT",
    "FOGSIG", "CRANES", "OFSPLF", "OSPARE", "PYLONS",
    "SISTAT", "SISTAW", "LITFLT", "LITVES",
    "SILTNK", "RADRFL", "RTPBCN", "RDOCAL", "CURENT",
    "NEWOBJ", "DISMAR", "MARCUL", "FSHFAC", "GATCON",
    "RETRFL", "PILBOP", "PILPNT",
];
