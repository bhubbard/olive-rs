use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PinDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PinType {
    Texture,
    Samples,
    Float,
    Color,
    Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pin {
    pub id: String,
    pub name: String,
    pub pin_type: PinType,
    pub direction: PinDirection,
}

impl Pin {
    pub fn input(id: impl Into<String>, name: impl Into<String>, pin_type: PinType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            pin_type,
            direction: PinDirection::Input,
        }
    }

    pub fn output(id: impl Into<String>, name: impl Into<String>, pin_type: PinType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            pin_type,
            direction: PinDirection::Output,
        }
    }
}
