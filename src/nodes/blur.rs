use std::collections::HashMap;
use uuid::Uuid;

use crate::buffer::{RgbaColor, RgbaImage};
use crate::error::Result;
use crate::graph::node::{EvaluationContext, Node, NodeOutputData};
use crate::graph::pin::{Pin, PinType};
use crate::keyframe::KeyframeTrack;

#[derive(Debug)]
pub struct BlurNode {
    pub id: Uuid,
    pub name: String,
    pub radius: KeyframeTrack<f64>,
}

impl BlurNode {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            radius: KeyframeTrack::new("radius", 2.0),
        }
    }
}

impl Node for BlurNode {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn category(&self) -> &str {
        "filter"
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

        let radius = (self.radius.evaluate_at(ctx.time).round() as i32).max(0);
        if radius == 0 {
            let mut outs = HashMap::new();
            outs.insert("tex_out".into(), NodeOutputData::Texture(in_img));
            return Ok(outs);
        }

        let w = in_img.width as i32;
        let h = in_img.height as i32;
        let mut out_img = RgbaImage::new(in_img.width, in_img.height, RgbaColor::TRANSPARENT);

        // Box blur pass
        for y in 0..h {
            for x in 0..w {
                let mut sum_r = 0.0;
                let mut sum_g = 0.0;
                let mut sum_b = 0.0;
                let mut sum_a = 0.0;
                let mut samples = 0.0;

                for dy in -radius..=radius {
                    let ny = (y + dy).clamp(0, h - 1) as u32;
                    for dx in -radius..=radius {
                        let nx = (x + dx).clamp(0, w - 1) as u32;
                        let p = in_img.get_pixel(nx, ny);
                        sum_r += p.r;
                        sum_g += p.g;
                        sum_b += p.b;
                        sum_a += p.a;
                        samples += 1.0;
                    }
                }

                out_img.set_pixel(
                    x as u32,
                    y as u32,
                    RgbaColor::new(sum_r / samples, sum_g / samples, sum_b / samples, sum_a / samples),
                );
            }
        }

        let mut outs = HashMap::new();
        outs.insert("tex_out".into(), NodeOutputData::Texture(out_img));
        Ok(outs)
    }
}
