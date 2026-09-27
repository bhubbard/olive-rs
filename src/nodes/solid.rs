use std::collections::HashMap;
use uuid::Uuid;

use crate::buffer::{RgbaColor, RgbaImage};
use crate::error::Result;
use crate::graph::node::{EvaluationContext, Node, NodeOutputData};
use crate::graph::pin::{Pin, PinType};
use crate::keyframe::KeyframeTrack;

#[derive(Debug)]
pub struct SolidGeneratorNode {
    pub id: Uuid,
    pub name: String,
    pub color_track_r: KeyframeTrack<f64>,
    pub color_track_g: KeyframeTrack<f64>,
    pub color_track_b: KeyframeTrack<f64>,
    pub color_track_a: KeyframeTrack<f64>,
}

impl SolidGeneratorNode {
    pub fn new(name: impl Into<String>, color: RgbaColor) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            color_track_r: KeyframeTrack::new("r", color.r as f64),
            color_track_g: KeyframeTrack::new("g", color.g as f64),
            color_track_b: KeyframeTrack::new("b", color.b as f64),
            color_track_a: KeyframeTrack::new("a", color.a as f64),
        }
    }
}

impl Node for SolidGeneratorNode {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn category(&self) -> &str {
        "generator"
    }

    fn inputs(&self) -> Vec<Pin> {
        vec![]
    }

    fn outputs(&self) -> Vec<Pin> {
        vec![Pin::output("tex_out", "Texture", PinType::Texture)]
    }

    fn evaluate(
        &self,
        ctx: &EvaluationContext,
        _inputs: &HashMap<String, NodeOutputData>,
    ) -> Result<HashMap<String, NodeOutputData>> {
        let r = self.color_track_r.evaluate_at(ctx.time) as f32;
        let g = self.color_track_g.evaluate_at(ctx.time) as f32;
        let b = self.color_track_b.evaluate_at(ctx.time) as f32;
        let a = self.color_track_a.evaluate_at(ctx.time) as f32;

        let img = RgbaImage::new(ctx.width, ctx.height, RgbaColor::new(r, g, b, a));

        let mut outs = HashMap::new();
        outs.insert("tex_out".into(), NodeOutputData::Texture(img));
        Ok(outs)
    }
}
