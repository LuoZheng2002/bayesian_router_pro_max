use std::ops::{Add, Div, Mul, Neg, Sub};

use serde::{Deserialize, Serialize};

pub type FixedPoint = fixed::types::I16F16;

#[derive(Debug, Clone, PartialEq, Hash, Eq, Copy, PartialOrd, Ord)]
pub struct FixedVec2 {
    pub x: FixedPoint,
    pub y: FixedPoint,
}

impl FixedVec2 {
    pub fn new(x: FixedPoint, y: FixedPoint) -> Self {
        FixedVec2 { x, y }
    }
    pub fn to_float(&self) -> FloatVec2 {
        FloatVec2 {
            x: self.x.to_num(),
            y: self.y.to_num(),
        }
    }
    pub fn length(&self) -> FixedPoint {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn is_x_odd_y_odd(&self) -> bool {
        self.x.to_bits() & 1 == 1 && self.y.to_bits() & 1 == 1
    }
    pub fn is_sum_even(&self) -> bool {
        (self.x.to_bits() + self.y.to_bits()) % 2 == 0
    }
    pub fn to_nearest_even_even(&self) -> FixedVec2 {
        let x_is_odd = self.x.to_bits() & 1 == 1;
        let y_is_odd = self.y.to_bits() & 1 == 1;
        if x_is_odd && y_is_odd {
            FixedVec2 {
                x: self.x - FixedPoint::DELTA,
                y: self.y - FixedPoint::DELTA,
            }
        } else if x_is_odd {
            FixedVec2 {
                x: self.x - FixedPoint::DELTA,
                y: self.y,
            }
        } else if y_is_odd {
            FixedVec2 {
                x: self.x,
                y: self.y - FixedPoint::DELTA,
            }
        } else {
            *self
        }
    }
}

impl Mul<FixedPoint> for FixedVec2 {
    type Output = FixedVec2;

    fn mul(self, scalar: FixedPoint) -> FixedVec2 {
        FixedVec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Div<FixedPoint> for FixedVec2 {
    type Output = FixedVec2;

    fn div(self, scalar: FixedPoint) -> FixedVec2 {
        if scalar == FixedPoint::ZERO {
            panic!("Division by zero in FixedVec2");
        }
        FixedVec2 {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl Sub for FixedVec2 {
    type Output = FixedVec2;

    fn sub(self, other: FixedVec2) -> FixedVec2 {
        FixedVec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl Add for FixedVec2 {
    type Output = FixedVec2;

    fn add(self, other: FixedVec2) -> FixedVec2 {
        FixedVec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Neg for FixedVec2 {
    type Output = FixedVec2;

    fn neg(self) -> FixedVec2 {
        FixedVec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct FloatVec2 {
    pub x: f32,
    pub y: f32,
}

impl FloatVec2 {
    pub fn new(x: f32, y: f32) -> Self {
        FloatVec2 { x, y }
    }
    pub fn to_fixed(&self) -> FixedVec2 {
        FixedVec2 {
            x: FixedPoint::from_num(self.x),
            y: FixedPoint::from_num(self.y),
        }
    }
    pub fn dot(self, other: FloatVec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Returns a vector perpendicular to self (normal to edge)
    pub fn perp(self) -> FloatVec2 {
        FloatVec2 {
            x: -self.y,
            y: self.x,
        }
    }

    /// Normalize the vector (used to prevent numerical issues)
    pub fn normalize(self) -> FloatVec2 {
        let len = (self.x * self.x + self.y * self.y).sqrt();
        if len > f32::EPSILON {
            FloatVec2 {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            self
        }
    }
    pub fn magnitude2(self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

impl Add for FloatVec2 {
    type Output = FloatVec2;

    fn add(self, other: FloatVec2) -> FloatVec2 {
        FloatVec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Sub for FloatVec2 {
    type Output = FloatVec2;

    fn sub(self, other: FloatVec2) -> FloatVec2 {
        FloatVec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Div<f32> for FloatVec2 {
    type Output = FloatVec2;

    fn div(self, scalar: f32) -> FloatVec2 {
        if scalar == 0.0 {
            panic!("Division by zero in FloatVec2");
        }
        FloatVec2 {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub struct IntVec2{
    pub x: i32,
    pub y: i32,
}

impl IntVec2 {
    pub fn new(x: i32, y: i32) -> Self {
        IntVec2 { x, y }
    }    
    pub fn to_fixed(&self) -> FixedVec2 {
        FixedVec2 {
            x: FixedPoint::from_num(self.x),
            y: FixedPoint::from_num(self.y),
        }
    }
}