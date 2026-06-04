//! Simple fixed-style layer implementations for layers that don't need conditional logic.

use crate::config::StyleConfig;
use crate::feature::PropMap;
use crate::style::FeatureStyle;

macro_rules! simple_layer {
    ($name:ident, area_color: $ac:expr) => {
        pub fn $name(_props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
            FeatureStyle {
                area_color: Some($ac.to_string()),
                ..Default::default()
            }
        }
    };
    ($name:ident, line_color: $lc:expr) => {
        pub fn $name(_props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
            FeatureStyle {
                line_color: Some($lc.to_string()),
                ..Default::default()
            }
        }
    };
    ($name:ident, symbol: $sy:expr) => {
        pub fn $name(_props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
            FeatureStyle {
                symbol: Some($sy.to_string()),
                ..Default::default()
            }
        }
    };
    ($name:ident, area_color: $ac:expr, line_color: $lc:expr) => {
        pub fn $name(_props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
            FeatureStyle {
                area_color: Some($ac.to_string()),
                line_color: Some($lc.to_string()),
                ..Default::default()
            }
        }
    };
    ($name:ident, symbol: $sy:expr, area_color: $ac:expr) => {
        pub fn $name(_props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
            FeatureStyle {
                symbol: Some($sy.to_string()),
                area_color: Some($ac.to_string()),
                ..Default::default()
            }
        }
    };
    ($name:ident, area_color: $ac:expr, area_pattern: $ap:expr) => {
        pub fn $name(_props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
            FeatureStyle {
                area_color: Some($ac.to_string()),
                area_pattern: Some($ap.to_string()),
                ..Default::default()
            }
        }
    };
}

simple_layer!(airare, area_color: "LANDA", line_color: "CHBLK");
simple_layer!(achare, line_color: "CHBRN");
simple_layer!(achbrt, symbol: "ACHBRT07");
simple_layer!(achpnt, symbol: "ACHARE02");
simple_layer!(berths, line_color: "CHGRD");
simple_layer!(boyinb, symbol: "BOYINB11");
simple_layer!(bridge, area_color: "CHBRN", line_color: "CHBLK");
simple_layer!(buaare, area_color: "LANDA");
simple_layer!(buisgl, area_color: "CHBRN", line_color: "CHBLK");
simple_layer!(canals, area_color: "DEPDW", line_color: "CHBLK");
simple_layer!(causwy, area_color: "CHBRN", line_color: "CHBLK");
simple_layer!(cblare, line_color: "CHGRD");
simple_layer!(cblohd, line_color: "CHBLK");
simple_layer!(cblsub, line_color: "CHBLK");
simple_layer!(cgusta, symbol: "CGUSTA01");
simple_layer!(chkpnt, symbol: "CHKPNT01");
simple_layer!(convyr, line_color: "CHBLK");
simple_layer!(cranes, symbol: "CRANES01");
simple_layer!(ctnare, line_color: "CHMGD");
simple_layer!(ctsare, line_color: "TRFCD");
simple_layer!(curent, symbol: "CURENT01");
simple_layer!(damcon, area_color: "CHBRN", line_color: "CHBLK");
simple_layer!(dismar, symbol: "DISMAR01");
simple_layer!(dmpgrd, line_color: "CHGRD");
simple_layer!(docare, area_color: "DEPDW");
simple_layer!(drgare, area_color: "NODTA");
simple_layer!(dwrtcl, line_color: "TRFCD");
simple_layer!(dwrtpt, area_color: "TRFCF", line_color: "TRFCD");
simple_layer!(dykcon, area_color: "CHBRN", line_color: "CHBLK");
simple_layer!(fairwy, area_color: "TRFCF", line_color: "TRFCD");
simple_layer!(fnclne, line_color: "CHBLK");
simple_layer!(feryrt, line_color: "CHBLK");
simple_layer!(flodoc, area_color: "CHBRN");
simple_layer!(fogsig, symbol: "FOGSIG03");
simple_layer!(forstc, area_color: "LANDA");
simple_layer!(fshfac, symbol: "FSHFAC03");
simple_layer!(fshgrd, line_color: "CHBLK");
simple_layer!(gatcon, symbol: "GATCON01");
simple_layer!(iceare, area_color: "CHWHT");
simple_layer!(icnare, line_color: "CHBLK");
simple_layer!(istzne, line_color: "TRFCD");
simple_layer!(lakare, area_color: "DEPDW");
simple_layer!(lakshr, line_color: "CSTLN");
simple_layer!(litflt, symbol: "LITFLT01");
simple_layer!(litves, symbol: "LITVES01");
simple_layer!(lndrgn, area_color: "LANDA");
simple_layer!(logpon, area_color: "DEPDW");
simple_layer!(lokbsn, area_color: "DEPDW");
simple_layer!(marcul, symbol: "MARCUL01");
simple_layer!(mipare, line_color: "CHRED");
simple_layer!(morfac, symbol: "MORFAC01");
simple_layer!(navlne, line_color: "CHBLK");
simple_layer!(newobj, symbol: "NEWOBJ02");
simple_layer!(ofsplf, symbol: "OFSPLF01");
simple_layer!(oilbar, line_color: "CHBLK");
simple_layer!(ospare, symbol: "OSPARE01");
simple_layer!(pilbop, symbol: "PILBOP01");
simple_layer!(pilpnt, symbol: "PILPNT01");
simple_layer!(pipare, line_color: "CHBLK");
simple_layer!(ponton, area_color: "CHBRN", line_color: "CHBLK");
simple_layer!(prcare, line_color: "CHYLW");
simple_layer!(prdare, area_color: "CHBRN");
simple_layer!(pylons, symbol: "PYLONS01");
simple_layer!(radlne, line_color: "RADHI");
simple_layer!(radrfl, symbol: "RADRFL01");
simple_layer!(radrng, line_color: "RADHI");
simple_layer!(rcrtcl, line_color: "CHBLK");
simple_layer!(rctlpt, area_color: "TRFCF", line_color: "TRFCD");
simple_layer!(rdocal, symbol: "RDOCAL01");
simple_layer!(rectrc, area_color: "TRFCF", line_color: "TRFCD");
simple_layer!(retrfl, symbol: "RETRFL01");
simple_layer!(rivers, line_color: "DEPDW");
simple_layer!(rtpbcn, symbol: "RTPBCN01");
simple_layer!(runway, area_color: "LANDA", line_color: "CHBLK");
simple_layer!(seaare, line_color: "CHGRD");
simple_layer!(siltnk, symbol: "SILTNK02");
simple_layer!(sistat, symbol: "SISTAT01");
simple_layer!(sistaw, symbol: "SISTAW01");
simple_layer!(slcons, line_color: "CSTLN");
simple_layer!(slogrd, area_color: "LANDA");

pub fn sndwav(_props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
    FeatureStyle {
        area_pattern: Some("SNDWAV01P".to_string()),
        ..Default::default()
    }
}

simple_layer!(splare, line_color: "TRFCD");
simple_layer!(subtln, line_color: "CHBLK");
simple_layer!(swpare, line_color: "TRFCD");
simple_layer!(tctlpt, line_color: "TRFCD");
simple_layer!(tselne, line_color: "TRFCD");
simple_layer!(tsebnd, line_color: "TRFCD");
simple_layer!(tsscrs, area_color: "TRFCF", line_color: "TRFCD");
simple_layer!(tsslpt, area_color: "TRFCF", line_color: "TRFCD");
simple_layer!(tssron, area_color: "TRFCF", line_color: "TRFCD");
simple_layer!(tsezne, area_color: "TRFCF", line_color: "TRFCD");
simple_layer!(tssbnd, line_color: "TRFCD");
simple_layer!(tunnel, line_color: "CHBLK");
simple_layer!(twrtpt, area_color: "TRFCF", line_color: "TRFCD");
simple_layer!(unsare, line_color: "CHGRD");
simple_layer!(wedklp, area_color: "DEPIT");
