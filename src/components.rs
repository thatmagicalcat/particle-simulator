use glam::DVec2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mass(pub f64);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position(pub DVec2);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Velocity(pub DVec2);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub usize);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Disabled;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MouseTracker {
    pub pos: DVec2,
    pub radius: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeInfo {
    pub radius: f64,
    pub color: sfml::graphics::Color,
}

