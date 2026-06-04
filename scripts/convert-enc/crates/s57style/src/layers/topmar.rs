use crate::config::StyleConfig;
use crate::feature::{PropMap, PropMapExt};
use crate::style::FeatureStyle;
use super::attr::topshp::{topshp, Topshp};

pub fn style_feature(props: &PropMap, _config: &StyleConfig) -> FeatureStyle {
    let mut style = FeatureStyle::default();

    // Read platform from "_PTFM" key in props (set by upstream processing)
    let is_floating = props.get_str("_PTFM").map_or(false, |v| v == "FLOATING");

    let sym = if is_floating {
        match topshp(props) {
            Some(Topshp::ConePointUp) => "TOPMAR02",
            Some(Topshp::ConePointDown) => "TOPMAR04",
            Some(Topshp::Sphere) => "TOPMAR10",
            Some(Topshp::TwoSphere) => "TOPMAR12",
            Some(Topshp::CylinderCan) => "TOPMAR13",
            Some(Topshp::Board) => "TOPMAR14",
            Some(Topshp::XShape) => "TOPMAR65",
            Some(Topshp::UprightCross) => "TOPMAR17",
            Some(Topshp::CubePointUp) => "TOPMAR16",
            Some(Topshp::TwoConesPointToPoint) => "TOPMAR08",
            Some(Topshp::TwoConesBaseToBase) => "TOPMAR07",
            Some(Topshp::RhombusDiamond) => "TOPMAR14",
            Some(Topshp::TwoConesPointsUpward) => "TOPMAR05",
            Some(Topshp::TwoConesPointsDownward) => "TOPMAR06",
            Some(Topshp::BesomPointUp) => "TMARDEF2",
            Some(Topshp::BesomPointDown) => "TMARDEF2",
            Some(Topshp::Flag) => "TMARDEF2",
            Some(Topshp::SphereOverRhombus) => "TOPMAR10",
            Some(Topshp::Square) => "TOPMAR13",
            Some(Topshp::RectangleHorizontal) => "TOPMAR14",
            Some(Topshp::RectangleVertical) => "TOPMAR13",
            Some(Topshp::TrapeziumUp) => "TOPMAR14",
            Some(Topshp::TrapeziumDown) => "TOPMAR14",
            Some(Topshp::TrianglePointUp) => "TOPMAR02",
            Some(Topshp::TrianglePointDown) => "TOPMAR04",
            Some(Topshp::Circle) => "TOPMAR10",
            Some(Topshp::TwoCrossesOneOver) => "TOPMAR17",
            Some(Topshp::TShape) => "TOPMAR18",
            Some(Topshp::TriangleOverCircle) => "TOPMAR02",
            Some(Topshp::CrossOverCircle) => "TOPMAR17",
            Some(Topshp::RhombusOverCircle) => "TOPMAR14",
            Some(Topshp::CircleOverTriangle) => "TOPMAR10",
            Some(Topshp::Other) | None => "TMARDEF2",
        }
    } else {
        // RIGID
        match topshp(props) {
            Some(Topshp::ConePointUp) => "TOPMAR22",
            Some(Topshp::ConePointDown) => "TOPMAR24",
            Some(Topshp::Sphere) => "TOPMAR30",
            Some(Topshp::TwoSphere) => "TOPMAR32",
            Some(Topshp::CylinderCan) => "TOPMAR33",
            Some(Topshp::Board) => "TOPMAR34",
            Some(Topshp::XShape) => "TOPMAR85",
            Some(Topshp::UprightCross) => "TOPMAR86",
            Some(Topshp::CubePointUp) => "TOPMAR36",
            Some(Topshp::TwoConesPointToPoint) => "TOPMAR28",
            Some(Topshp::TwoConesBaseToBase) => "TOPMAR27",
            Some(Topshp::RhombusDiamond) => "TOPMAR14",
            Some(Topshp::TwoConesPointsUpward) => "TOPMAR25",
            Some(Topshp::TwoConesPointsDownward) => "TOPMAR26",
            Some(Topshp::BesomPointUp) => "TOPMAR88",
            Some(Topshp::BesomPointDown) => "TOPMAR87",
            Some(Topshp::Flag) => "TMARDEF1",
            Some(Topshp::SphereOverRhombus) => "TOPMAR30",
            Some(Topshp::Square) => "TOPMAR33",
            Some(Topshp::RectangleHorizontal) => "TOPMAR34",
            Some(Topshp::RectangleVertical) => "TOPMAR33",
            Some(Topshp::TrapeziumUp) => "TOPMAR34",
            Some(Topshp::TrapeziumDown) => "TOPMAR34",
            Some(Topshp::TrianglePointUp) => "TOPMAR22",
            Some(Topshp::TrianglePointDown) => "TOPMAR24",
            Some(Topshp::Circle) => "TOPMAR30",
            Some(Topshp::TwoCrossesOneOver) => "TOPMAR86",
            Some(Topshp::TShape) => "TOPMAR89",
            Some(Topshp::TriangleOverCircle) => "TOPMAR22",
            Some(Topshp::CrossOverCircle) => "TOPMAR86",
            Some(Topshp::RhombusOverCircle) => "TOPMAR14",
            Some(Topshp::CircleOverTriangle) => "TOPMAR30",
            Some(Topshp::Other) | None => "TMARDEF1",
        }
    };

    style.symbol = Some(sym.to_string());
    style
}
