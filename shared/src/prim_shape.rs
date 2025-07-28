use cgmath::{Rotation, Rotation2};
use serde::{Deserialize, Serialize};

use crate::vec2::FloatVec2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircleShape {
    pub position: FloatVec2,
    pub diameter: f32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RectangleShape {
    pub position: FloatVec2, // center position of the rectangle
    pub width: f32,
    pub height: f32,
    pub rotation_in_degs: f32, // Rotation counterclockwise in degrees
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub start: FloatVec2,
    pub end: FloatVec2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimShape {
    Circle(CircleShape),
    Rectangle(RectangleShape),
    Line(Line),
}
