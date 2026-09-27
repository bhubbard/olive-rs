use num_rational::Rational64;
use uuid::Uuid;

use crate::timeline::block::Block;
use crate::timeline::track::{Track, TrackList};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrimMode {
    TrimIn,
    TrimOut,
}

/// Trims a block's In or Out point.
pub struct BlockTrimCommand {
    block_id: Uuid,
    new_length: Rational64,
    mode: TrimMode,
    saved_blocks: Option<Vec<Block>>,
}

impl BlockTrimCommand {
    pub fn new(block_id: Uuid, new_length: Rational64, mode: TrimMode) -> Self {
        Self {
            block_id,
            new_length,
            mode,
            saved_blocks: None,
        }
    }

    pub fn redo(&mut self, track: &mut Track) {
        if self.saved_blocks.is_none() {
            self.saved_blocks = Some(track.blocks.clone());
        }

        let idx = match track.blocks.iter().position(|b| b.id == self.block_id) {
            Some(i) => i,
            None => return,
        };

        let old_len = track.blocks[idx].length;
        let delta = old_len - self.new_length;

        match self.mode {
            TrimMode::TrimOut => {
                track.blocks[idx].length = self.new_length;
                track.blocks[idx].media_out = track.blocks[idx].media_in + self.new_length;

                // If not at the end of the track, insert a gap of size delta
                if idx + 1 < track.blocks.len() && delta > Rational64::new(0, 1) {
                    let gap = Block::new_gap(delta);
                    track.blocks.insert(idx + 1, gap);
                }
            }
            TrimMode::TrimIn => {
                track.blocks[idx].length = self.new_length;
                track.blocks[idx].media_in += delta;

                // Insert a gap before this block
                if delta > Rational64::new(0, 1) {
                    let gap = Block::new_gap(delta);
                    track.blocks.insert(idx, gap);
                }
            }
        }
    }

    pub fn undo(&mut self, track: &mut Track) {
        if let Some(ref saved) = self.saved_blocks {
            track.blocks = saved.clone();
        }
    }
}

/// Replaces a block with a gap and handles adjacent/trailing gap consolidation
/// and transition updates.
pub struct TrackReplaceBlockWithGapCommand {
    block_id: Uuid,
    saved_blocks: Option<Vec<Block>>,
}

impl TrackReplaceBlockWithGapCommand {
    pub fn new(block_id: Uuid) -> Self {
        Self {
            block_id,
            saved_blocks: None,
        }
    }

    pub fn redo(&mut self, track: &mut Track) {
        if self.saved_blocks.is_none() {
            self.saved_blocks = Some(track.blocks.clone());
        }

        let idx = match track.blocks.iter().position(|b| b.id == self.block_id) {
            Some(i) => i,
            None => return,
        };

        let target_len = track.blocks[idx].length;
        let is_last = idx == track.blocks.len() - 1;

        if is_last {
            // At the end of the track: simply remove it, no trailing gap
            track.blocks.remove(idx);
            track.consolidate_gaps();
            return;
        }

        let removed_block_id = self.block_id;

        // Disconnect or update transitions pointing to this block
        for b in track.blocks.iter_mut() {
            if b.is_transition() {
                if b.in_block == Some(removed_block_id) {
                    b.in_block = None;
                }
                if b.out_block == Some(removed_block_id) {
                    b.out_block = None;
                }
            }
        }

        // Replace block with a gap
        let gap = Block::new_gap(target_len);
        track.blocks[idx] = gap;

        // Clean up orphaned transitions
        let mut i = 0;
        while i < track.blocks.len() {
            if track.blocks[i].is_transition()
                && track.blocks[i].in_block.is_none()
                && track.blocks[i].out_block.is_none()
            {
                let t_len = track.blocks[i].length;
                track.blocks[i] = Block::new_gap(t_len);
            }
            i += 1;
        }

        track.consolidate_gaps();
    }

    pub fn undo(&mut self, track: &mut Track) {
        if let Some(ref saved) = self.saved_blocks {
            track.blocks = saved.clone();
        }
    }
}

/// Inserts gaps across tracks in a TrackList at a given time offset.
pub struct TrackListInsertGaps {
    time: Rational64,
    gap_length: Rational64,
    saved_tracks: Option<Vec<Track>>,
}

impl TrackListInsertGaps {
    pub fn new(time: Rational64, gap_length: Rational64) -> Self {
        Self {
            time,
            gap_length,
            saved_tracks: None,
        }
    }

    pub fn redo(&mut self, track_list: &mut TrackList) {
        if self.saved_tracks.is_none() {
            self.saved_tracks = Some(track_list.tracks.clone());
        }

        for track in &mut track_list.tracks {
            let total_len = track.total_length();
            if self.time >= total_len {
                continue;
            }

            if self.time == Rational64::new(0, 1) {
                let gap = Block::new_gap(self.gap_length);
                track.blocks.insert(0, gap);
                continue;
            }

            let mut cur_time = Rational64::new(0, 1);
            let mut insert_idx = None;
            let mut split_info = None;

            for (idx, b) in track.blocks.iter().enumerate() {
                let next_time = cur_time + b.length;
                if self.time == cur_time {
                    insert_idx = Some(idx);
                    break;
                } else if self.time > cur_time && self.time < next_time {
                    let first_part_len = self.time - cur_time;
                    let second_part_len = b.length - first_part_len;
                    split_info = Some((idx, first_part_len, second_part_len));
                    break;
                }
                cur_time = next_time;
            }

            if let Some((idx, len1, len2)) = split_info {
                let original = &track.blocks[idx];
                let mut b1 = original.clone();
                b1.set_length_and_media_out(len1);

                let mut b2 = original.clone();
                b2.id = Uuid::new_v4();
                b2.media_in += len1;
                b2.set_length_and_media_out(len2);

                let gap = Block::new_gap(self.gap_length);

                track.blocks[idx] = b1;
                track.blocks.insert(idx + 1, gap);
                track.blocks.insert(idx + 2, b2);
            } else if let Some(idx) = insert_idx {
                let gap = Block::new_gap(self.gap_length);
                track.blocks.insert(idx, gap);
            }
        }
    }

    pub fn undo(&mut self, track_list: &mut TrackList) {
        if let Some(ref saved) = self.saved_tracks {
            track_list.tracks = saved.clone();
        }
    }
}
