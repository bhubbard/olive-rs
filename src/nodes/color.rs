use std::collections::HashMap;
use uuid::Uuid;

use crate::buffer::{RgbaColor, RgbaImage};
use crate::error::Result;
use crate::graph::node::{EvaluationContext, Node, NodeOutputData};
use crate::graph::pin::{Pin, PinType};
use crate::keyframe::KeyframeTrack;

#[derive(Debug)]
pub struct ColorGradeNode {
    pub id: Uuid,
    pub name: String,
    pub brightness: KeyframeTrack<f64>,
    pub contrast: KeyframeTrack<f64>,
    pub saturation: KeyframeTrack<f64>,
    pub gamma: KeyframeTrack<f64>,
}

impl ColorGradeNode {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            brightness: KeyframeTrack::new("brightness", 0.0),
            contrast: KeyframeTrack::new("contrast", 1.0),
            saturation: KeyframeTrack::new("saturation", 1.0),
            gamma: KeyframeTrack::new("gamma", 1.0),
        }
    }
}

impl Node for ColorGradeNode {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn category(&self) -> &str {
        "color"
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

        let brightness = self.brightness.evaluate_at(ctx.time) as f32;
        let contrast = self.contrast.evaluate_at(ctx.time) as f32;
        let saturation = self.saturation.evaluate_at(ctx.time) as f32;
        let gamma = (self.gamma.evaluate_at(ctx.time) as f32).max(0.01);
        let inv_gamma = 1.0 / gamma;

        let w = in_img.width;
        let h = in_img.height;
        let mut out_img = RgbaImage::new(w, h, RgbaColor::TRANSPARENT);

        for y in 0..h {
            for x in 0..w {
                let c = in_img.get_pixel(x, y);

                // Brightness & Contrast
                let mut r = (c.r - 0.5) * contrast + 0.5 + brightness;
                let mut g = (c.g - 0.5) * contrast + 0.5 + brightness;
                let mut b = (c.b - 0.5) * contrast + 0.5 + brightness;

                // Saturation
                let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                r = luma + (r - luma) * saturation;
                g = luma + (g - luma) * saturation;
                b = luma + (b - luma) * saturation;

                // Gamma
                r = r.max(0.0).powf(inv_gamma);
                g = g.max(0.0).powf(inv_gamma);
                b = b.max(0.0).powf(inv_gamma);

                out_img.set_pixel(x, y, RgbaColor::new(r, g, b, c.a));
            }
        }

        let mut outs = HashMap::new();
        outs.insert("tex_out".into(), NodeOutputData::Texture(out_img));
        Ok(outs)
    }
}
