use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::timeline::block::Block;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrackType {
    Video,
    Audio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: Uuid,
    pub name: String,
    pub track_type: TrackType,
    pub blocks: Vec<Block>,
    pub mute: bool,
    pub lock: bool,
}

impl Track {
    pub fn new(name: impl Into<String>, track_type: TrackType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            track_type,
            blocks: Vec::new(),
            mute: false,
            lock: false,
        }
    }

    pub fn append_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn total_length(&self) -> Rational64 {
        self.blocks.iter().map(|b| b.length).sum()
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn blocks_mut(&mut self) -> &mut Vec<Block> {
        &mut self.blocks
    }

    pub fn consolidate_gaps(&mut self) {
        // Remove trailing gaps at the end of the track
        while let Some(last) = self.blocks.last() {
            if last.is_gap() {
                self.blocks.pop();
            } else {
                break;
            }
        }

        // Merge adjacent gaps
        let mut i = 0;
        while i + 1 < self.blocks.len() {
            if self.blocks[i].is_gap() && self.blocks[i + 1].is_gap() {
                let next_len = self.blocks[i + 1].length;
                self.blocks[i].length += next_len;
                self.blocks[i].media_out += next_len;
                self.blocks.remove(i + 1);
            } else {
                i += 1;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackList {
    pub track_type: TrackType,
    pub tracks: Vec<Track>,
}

impl TrackList {
    pub fn new(track_type: TrackType) -> Self {
        Self {
            track_type,
            tracks: Vec::new(),
        }
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    pub fn get_track_count(&self) -> usize {
        self.tracks.len()
    }

    pub fn get_track(&self, index: usize) -> Option<&Track> {
        self.tracks.get(index)
    }

    pub fn get_track_mut(&mut self, index: usize) -> Option<&mut Track> {
        self.tracks.get_mut(index)
    }
}
