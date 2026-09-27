use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockKind {
    Clip,
    Gap,
    Transition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: Uuid,
    pub kind: BlockKind,
    pub name: String,
    pub length: Rational64,
    pub media_in: Rational64,
    pub media_out: Rational64,
    // Connected block IDs for transitions (e.g. InBlock, OutBlock)
    pub in_block: Option<Uuid>,
    pub out_block: Option<Uuid>,
}

impl Block {
    pub fn new_clip(name: impl Into<String>, length: Rational64) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind: BlockKind::Clip,
            name: name.into(),
            length,
            media_in: Rational64::new(0, 1),
            media_out: length,
            in_block: None,
            out_block: None,
        }
    }

    pub fn new_gap(length: Rational64) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind: BlockKind::Gap,
            name: "Gap".into(),
            length,
            media_in: Rational64::new(0, 1),
            media_out: length,
            in_block: None,
            out_block: None,
        }
    }

    pub fn new_transition(name: impl Into<String>, length: Rational64) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind: BlockKind::Transition,
            name: name.into(),
            length,
            media_in: Rational64::new(0, 1),
            media_out: length,
            in_block: None,
            out_block: None,
        }
    }

    pub fn is_clip(&self) -> bool {
        self.kind == BlockKind::Clip
    }

    pub fn is_gap(&self) -> bool {
        self.kind == BlockKind::Gap
    }

    pub fn is_transition(&self) -> bool {
        self.kind == BlockKind::Transition
    }

    pub fn set_length_and_media_out(&mut self, len: Rational64) {
        self.length = len;
        self.media_out = self.media_in + len;
    }
}
