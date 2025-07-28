use serde::{Deserialize, Serialize};



#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SettingsEnum{
    Bool(bool),
    Usize(usize),
    Float(f64),
}

impl SettingsEnum {
    pub fn as_bool(&self) -> Option<bool> {
        if let SettingsEnum::Bool(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_usize(&self) -> Option<usize> {
        if let SettingsEnum::Usize(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        if let SettingsEnum::Float(value) = self {
            Some(*value)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSettingsArg{
    pub setting: String,
}

impl GetSettingsArg {
    pub fn new(setting: String) -> Self {
        GetSettingsArg { setting }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSettingsArg{
    pub setting: String,
    pub value: SettingsEnum,
}

impl SetSettingsArg {
    pub fn new(setting: String, value: SettingsEnum) -> Self {
        SetSettingsArg { setting, value }
    }
}