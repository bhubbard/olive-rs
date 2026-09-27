use std::collections::HashMap;
use uuid::Uuid;

use crate::buffer::{RgbaColor, RgbaImage};
use crate::error::Result;
use crate::graph::node::{EvaluationContext, Node, NodeOutputData};
use crate::graph::pin::{Pin, PinType};
use crate::keyframe::KeyframeTrack;

#[derive(Debug)]
pub struct TransformDistortNode {
    pub id: Uuid,
    pub name: String,
    pub pos_x: KeyframeTrack<f64>,
    pub pos_y: KeyframeTrack<f64>,
    pub scale_x: KeyframeTrack<f64>,
    pub scale_y: KeyframeTrack<f64>,
    pub rotation: KeyframeTrack<f64>,
}

impl TransformDistortNode {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            pos_x: KeyframeTrack::new("pos_x", 0.0),
            pos_y: KeyframeTrack::new("pos_y", 0.0),
            scale_x: KeyframeTrack::new("scale_x", 1.0),
            scale_y: KeyframeTrack::new("scale_y", 1.0),
            rotation: KeyframeTrack::new("rotation", 0.0),
        }
    }
}

impl Node for TransformDistortNode {
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

        let dx = self.pos_x.evaluate_at(ctx.time);
        let dy = self.pos_y.evaluate_at(ctx.time);
        let sx = self.scale_x.evaluate_at(ctx.time);
        let sy = self.scale_y.evaluate_at(ctx.time);
        let rot_deg = self.rotation.evaluate_at(ctx.time);
        let rad = rot_deg.to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();

        let w = ctx.width;
        let h = ctx.height;
        let mut out_img = RgbaImage::new(w, h, RgbaColor::TRANSPARENT);

        let cx = (w as f64) / 2.0;
        let cy = (h as f64) / 2.0;

        let inv_sx = if sx.abs() < 1e-6 { 1.0 } else { 1.0 / sx };
        let inv_sy = if sy.abs() < 1e-6 { 1.0 } else { 1.0 / sy };

        for y in 0..h {
            for x in 0..w {
                // Backward mapping from destination to source
                let px = (x as f64) - cx - dx;
                let py = (y as f64) - cy - dy;

                let rx = px * cos_a + py * sin_a;
                let ry = -px * sin_a + py * cos_a;

                let src_x = (rx * inv_sx + cx).round() as i64;
                let src_y = (ry * inv_sy + cy).round() as i64;

                if src_x >= 0 && src_x < in_img.width as i64 && src_y >= 0 && src_y < in_img.height as i64 {
                    let c = in_img.get_pixel(src_x as u32, src_y as u32);
                    out_img.set_pixel(x, y, c);
                }
            }
        }

        let mut outs = HashMap::new();
        outs.insert("tex_out".into(), NodeOutputData::Texture(out_img));
        Ok(outs)
    }
}
