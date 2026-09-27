use std::collections::HashMap;
use uuid::Uuid;

use crate::buffer::{RgbaColor, RgbaImage};
use crate::error::Result;
use crate::graph::node::{EvaluationContext, Node, NodeOutputData};
use crate::graph::pin::{Pin, PinType};
use crate::keyframe::KeyframeTrack;

#[derive(Debug)]
pub struct CropDistortNode {
    pub id: Uuid,
    pub name: String,
    pub crop_left: KeyframeTrack<f64>,
    pub crop_top: KeyframeTrack<f64>,
    pub crop_right: KeyframeTrack<f64>,
    pub crop_bottom: KeyframeTrack<f64>,
}

impl CropDistortNode {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            crop_left: KeyframeTrack::new("crop_left", 0.0),
            crop_top: KeyframeTrack::new("crop_top", 0.0),
            crop_right: KeyframeTrack::new("crop_right", 0.0),
            crop_bottom: KeyframeTrack::new("crop_bottom", 0.0),
        }
    }
}

impl Node for CropDistortNode {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn category(&self) -> &str {
        "distort"
    }

    fn inputs(&self) -> Vec<Pin> {
        vec![Pin::input("tex_in", "Texture In", PinType::Texture)]
    }

    fn outputs(&self) -> Vec<Pin> {
        vec![Pin::output("tex_out", "Texture Out", PinType::Texture)]
    }

    fn evaluate(
        &self,
        ctx: &EvaluationContext,
        inputs: &HashMap<String, NodeOutputData>,
    ) -> Result<HashMap<String, NodeOutputData>> {
        let in_img = match inputs.get("tex_in") {
            Some(NodeOutputData::Texture(img)) => img.clone(),
            _ => RgbaImage::new(ctx.width, ctx.height, RgbaColor::TRANSPARENT),
        };

        let w = in_img.width;
        let h = in_img.height;
        let mut out_img = in_img.clone();

        // Crops as normalized fractions 0.0 to 1.0 or pixel amounts
        let cl = (self.crop_left.evaluate_at(ctx.time).clamp(0.0, 1.0) * w as f64) as u32;
        let ct = (self.crop_top.evaluate_at(ctx.time).clamp(0.0, 1.0) * h as f64) as u32;
        let cr = (self.crop_right.evaluate_at(ctx.time).clamp(0.0, 1.0) * w as f64) as u32;
        let cb = (self.crop_bottom.evaluate_at(ctx.time).clamp(0.0, 1.0) * h as f64) as u32;

        for y in 0..h {
            for x in 0..w {
                if x < cl || x >= (w.saturating_sub(cr)) || y < ct || y >= (h.saturating_sub(cb)) {
                    out_img.set_pixel(x, y, RgbaColor::TRANSPARENT);
                }
            }
        }

        let mut outs = HashMap::new();
        outs.insert("tex_out".into(), NodeOutputData::Texture(out_img));
        Ok(outs)
    }
}
