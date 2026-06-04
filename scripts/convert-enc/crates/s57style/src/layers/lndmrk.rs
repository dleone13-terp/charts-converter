use crate::config::StyleConfig;
use crate::feature::PropMap;
use crate::style::FeatureStyle;
use super::attr::catlmk::{catlmk, Catlmk};
use super::attr::convis::{convis, Convis};
use super::attr::functn::{functn, Functn};

pub fn style_feature(props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
    let mut style = FeatureStyle::default();

    let functn_list = functn(props);
    let viz = matches!(convis(props), Some(Convis::VisualConspicuous));

    if viz {
        style.area_color = Some("CHBRN".to_string());
        style.line_color = Some("CHBLK".to_string());
    } else {
        style.area_color = Some("CHBRN".to_string());
        style.line_color = Some("LANDF".to_string());
    }

    // Church/Chapel overrides
    if functn_list.iter().any(|f| matches!(f, Functn::Church | Functn::Chapel)) {
        style.symbol = Some(if viz { "BUIREL13" } else { "BUIREL01" }.to_string());
        return style;
    }

    // Mosque/Marabout overrides
    if functn_list.iter().any(|f| matches!(f, Functn::Mosque | Functn::Marabout)) {
        style.symbol = Some(if viz { "BUIREL15" } else { "BUIREL05" }.to_string());
        return style;
    }

    // CATLMK-based symbol
    let sym = match catlmk(props).into_iter().next() {
        Some(Catlmk::Cairn) => {
            if viz { "CAIRNS11" } else { "CAIRNS01" }
        }
        Some(Catlmk::Chimney) => {
            if viz { "CHIMNY11" } else { "CHIMNY01" }
        }
        Some(Catlmk::DishAerial) => "DSHAER11",
        Some(Catlmk::FlagstaffFlagpole) => "FLGSTF01",
        Some(Catlmk::FlareStack) => {
            if viz { "FLASTK11" } else { "FLASTK01" }
        }
        Some(Catlmk::Mast) => {
            if viz { "MSTCON14" } else { "MSTCON04" }
        }
        Some(Catlmk::Monument) => "MONUMT12",
        Some(Catlmk::ColumnPillar) | Some(Catlmk::Obelisk) | Some(Catlmk::Statue) => "MONUMT12",
        Some(Catlmk::Cross) => {
            if viz { "BUIREL13" } else { "BUIREL01" }
        }
        Some(Catlmk::Dome) => {
            if viz { "DOMES011" } else { "DOMES001" }
        }
        Some(Catlmk::RadarScanner) => {
            if viz { "RASCAN11" } else { "RASCAN01" }
        }
        Some(Catlmk::Tower) => {
            if viz { "TOWERS12" } else { "TOWERS02" }
        }
        Some(Catlmk::Windmill) => "WNDMIL12",
        Some(Catlmk::Windmotor) => {
            if viz { "WIMCON11" } else { "WIMCON01" }
        }
        Some(Catlmk::MemorialPlaque)
        | Some(Catlmk::Cemetery)
        | Some(Catlmk::Windsock)
        | Some(Catlmk::SpireMinaret)
        | None => {
            if viz { "POSGEN03" } else { "POSGEN01" }
        }
    };

    style.symbol = Some(sym.to_string());
    style
}
