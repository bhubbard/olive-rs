use std::collections::HashMap;
use std::sync::Arc;
use num_rational::Rational64;
use uuid::Uuid;

use crate::error::Result;
use crate::graph::dag::Graph;
use crate::graph::node::{EvaluationContext, Node, NodeOutputData};
use crate::graph::pin::{Pin, PinType};
use crate::nodes::math::{MathNode, MathOperation};
use crate::nodes::merge::{BlendMode, MergeNode};
use crate::timeline::track::{Track, TrackList, TrackType};

#[derive(Debug)]
pub struct SequenceNode {
    pub id: Uuid,
    pub name: String,
}

impl SequenceNode {
    pub const TEXTURE_INPUT: &'static str = "tex_in";
    pub const SAMPLES_INPUT: &'static str = "smp_in";
    pub const TEX_OUT: &'static str = "tex_out";
    pub const SMP_OUT: &'static str = "smp_out";

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
        }
    }
}

impl Node for SequenceNode {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn category(&self) -> &str {
        "project"
    }

    fn inputs(&self) -> Vec<Pin> {
        vec![
            Pin::input(Self::TEXTURE_INPUT, "Texture In", PinType::Texture),
            Pin::input(Self::SAMPLES_INPUT, "Samples In", PinType::Samples),
        ]
    }

    fn outputs(&self) -> Vec<Pin> {
        vec![
            Pin::output(Self::TEX_OUT, "Texture Out", PinType::Texture),
            Pin::output(Self::SMP_OUT, "Samples Out", PinType::Samples),
        ]
    }

    fn evaluate(
        &self,
        _ctx: &EvaluationContext,
        inputs: &HashMap<String, NodeOutputData>,
    ) -> Result<HashMap<String, NodeOutputData>> {
        let mut outs = HashMap::new();
        if let Some(t) = inputs.get(Self::TEXTURE_INPUT) {
            outs.insert(Self::TEX_OUT.into(), t.clone());
        }
        if let Some(s) = inputs.get(Self::SAMPLES_INPUT) {
            outs.insert(Self::SMP_OUT.into(), s.clone());
        }
        Ok(outs)
    }
}

pub struct Sequence {
    pub id: Uuid,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub frame_rate: Rational64,
    pub video_tracks: TrackList,
    pub audio_tracks: TrackList,
    pub graph: Graph,
    pub sequence_node_id: Uuid,
    pub connected_texture_output: Option<Uuid>,
    pub connected_samples_output: Option<Uuid>,
}

impl Sequence {
    pub fn new(name: impl Into<String>, width: u32, height: u32, frame_rate: Rational64) -> Self {
        let seq_node = Arc::new(SequenceNode::new("Sequence Output"));
        let seq_id = seq_node.id();

        let mut graph = Graph::new();
        graph.add_node(seq_node);

        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            width,
            height,
            frame_rate,
            video_tracks: TrackList::new(TrackType::Video),
            audio_tracks: TrackList::new(TrackType::Audio),
            graph,
            sequence_node_id: seq_id,
            connected_texture_output: None,
            connected_samples_output: None,
        }
    }

    pub fn track_list(&self, track_type: TrackType) -> &TrackList {
        match track_type {
            TrackType::Video => &self.video_tracks,
            TrackType::Audio => &self.audio_tracks,
        }
    }

    pub fn track_list_mut(&mut self, track_type: TrackType) -> &mut TrackList {
        match track_type {
            TrackType::Video => &mut self.video_tracks,
            TrackType::Audio => &mut self.audio_tracks,
        }
    }

    pub fn get_tracks(&self) -> Vec<&Track> {
        let mut list = Vec::new();
        for t in &self.video_tracks.tracks {
            list.push(t);
        }
        for t in &self.audio_tracks.tracks {
            list.push(t);
        }
        list
    }

    /// Adds initial default video and audio tracks and connects them to sequence
    pub fn add_default_nodes(&mut self) {
        let v_track = Track::new("Video 1", TrackType::Video);
        let a_track = Track::new("Audio 1", TrackType::Audio);

        let v_id = v_track.id;
        let a_id = a_track.id;

        self.video_tracks.add_track(v_track);
        self.audio_tracks.add_track(a_track);

        self.connected_texture_output = Some(v_id);
        self.connected_samples_output = Some(a_id);
    }

    /// Add a track to the sequence, optionally blending multi-track video with MergeNode
    /// and multi-track audio with MathNode.
    pub fn add_track(&mut self, track_type: TrackType, with_merge: bool) -> Uuid {
        let count = match track_type {
            TrackType::Video => self.video_tracks.get_track_count() + 1,
            TrackType::Audio => self.audio_tracks.get_track_count() + 1,
        };

        let name = match track_type {
            TrackType::Video => format!("Video {}", count),
            TrackType::Audio => format!("Audio {}", count),
        };

        let new_track = Track::new(name, track_type);
        let new_track_id = new_track.id;

        match track_type {
            TrackType::Video => {
                let existing_count = self.video_tracks.get_track_count();
                self.video_tracks.add_track(new_track);

                if existing_count == 0 {
                    // First video track connects directly to sequence texture input
                    self.connected_texture_output = Some(new_track_id);
                } else if with_merge {
                    // Create MergeNode to composite new track over existing tracks
                    let _prev_output = self.connected_texture_output.unwrap();
                    let merge = Arc::new(MergeNode::new("Track Merge", BlendMode::Normal));
                    let merge_id = merge.id();

                    self.graph.add_node(merge);
                    self.connected_texture_output = Some(merge_id);
                } else {
                    self.connected_texture_output = Some(new_track_id);
                }
            }
            TrackType::Audio => {
                let existing_count = self.audio_tracks.get_track_count();
                self.audio_tracks.add_track(new_track);

                if existing_count == 0 {
                    // First audio track connects directly to sequence samples input
                    self.connected_samples_output = Some(new_track_id);
                } else if with_merge {
                    // Create MathNode to mix audio tracks
                    let _prev_output = self.connected_samples_output.unwrap();
                    let math = Arc::new(MathNode::new("Audio Mix", MathOperation::Mix));
                    let math_id = math.id();

                    self.graph.add_node(math);
                    self.connected_samples_output = Some(math_id);
                } else {
                    self.connected_samples_output = Some(new_track_id);
                }
            }
        }

        new_track_id
    }
}
