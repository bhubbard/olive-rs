use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Result;
use crate::graph::connection::Connection;
use crate::timeline::block::Block;
use crate::timeline::track::TrackType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedTrack {
    pub id: Uuid,
    pub name: String,
    pub track_type: TrackType,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedSequence {
    pub id: Uuid,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub frame_rate_num: i64,
    pub frame_rate_den: i64,
    pub video_tracks: Vec<SerializedTrack>,
    pub audio_tracks: Vec<SerializedTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedNode {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub properties: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OliveProject {
    pub version: u32, // e.g. 230220
    pub generator: String,
    pub sequences: Vec<SerializedSequence>,
    pub nodes: Vec<SerializedNode>,
    pub connections: Vec<Connection>,
}

impl OliveProject {
    pub const CURRENT_VERSION: u32 = 230220;

    pub fn new() -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            generator: "olive-rs 0.0.1".into(),
            sequences: Vec::new(),
            nodes: Vec::new(),
            connections: Vec::new(),
        }
    }

    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}
