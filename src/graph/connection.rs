use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Connection {
    pub from_node: Uuid,
    pub from_pin: String,
    pub to_node: Uuid,
    pub to_pin: String,
}

impl Connection {
    pub fn new(
        from_node: Uuid,
        from_pin: impl Into<String>,
        to_node: Uuid,
        to_pin: impl Into<String>,
    ) -> Self {
        Self {
            from_node,
            from_pin: from_pin.into(),
            to_node,
            to_pin: to_pin.into(),
        }
    }
}
