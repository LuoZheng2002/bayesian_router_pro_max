use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatsArgs{
    pub stat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatsEnum{
    Float(f64),
    Usize(usize),
}

impl StatsEnum {
    pub fn as_float(&self) -> Option<f64> {
        if let StatsEnum::Float(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_usize(&self) -> Option<usize> {
        if let StatsEnum::Usize(value) = self {
            Some(*value)
        } else {
            None
        }
    }
}