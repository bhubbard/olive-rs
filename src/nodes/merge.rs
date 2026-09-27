use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::buffer::{RgbaColor, RgbaImage};
use crate::error::Result;
use crate::graph::node::{EvaluationContext, Node, NodeOutputData};
use crate::graph::pin::{Pin, PinType};
use crate::keyframe::KeyframeTrack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Add,
    Subtract,
    Darken,
    Lighten,
    Difference,
}

#[derive(Debug)]
pub struct MergeNode {
    pub id: Uuid,
    pub name: String,
    pub blend_mode: BlendMode,
    pub opacity: KeyframeTrack<f64>,
}

impl MergeNode {
    pub const BASE_IN: &'static str = "base_in";
    pub const BLEND_IN: &'static str = "blend_in";
    pub const TEX_OUT: &'static str = "tex_out";

    pub fn new(name: impl Into<String>, blend_mode: BlendMode) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            blend_mode,
            opacity: KeyframeTrack::new("opacity", 1.0),
        }
    }

    fn blend_channel(mode: BlendMode, b: f32, s: f32) -> f32 {
        match mode {
            BlendMode::Normal => s,
            BlendMode::Multiply => b * s,
            BlendMode::Screen => 1.0 - (1.0 - b) * (1.0 - s),
            BlendMode::Overlay => {
                if b < 0.5 {
                    2.0 * b * s
                } else {
                    1.0 - 2.0 * (1.0 - b) * (1.0 - s)
                }
            }
            BlendMode::Add => (b + s).min(1.0),
            BlendMode::Subtract => (b - s).max(0.0),
            BlendMode::Darken => b.min(s),
            BlendMode::Lighten => b.max(s),
            BlendMode::Difference => (b - s).abs(),
        }
    }
}

impl Node for MergeNode {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn category(&self) -> &str {
        "math"
    }

    fn inputs(&self) -> Vec<Pin> {
        vec![
            Pin::input(Self::BASE_IN, "Base In", PinType::Texture),
            Pin::input(Self::BLEND_IN, "Blend In", PinType::Texture),
        ]
    }

    fn outputs(&self) -> Vec<Pin> {
        vec![Pin::output(Self::TEX_OUT, "Texture Out", PinType::Texture)]
    }

    fn evaluate(
        &self,
        ctx: &EvaluationContext,
        inputs: &HashMap<String, NodeOutputData>,
    ) -> Result<HashMap<String, NodeOutputData>> {
        let base_img = match inputs.get(Self::BASE_IN) {
            Some(NodeOutputData::Texture(img)) => img.clone(),
            _ => RgbaImage::new(ctx.width, ctx.height, RgbaColor::TRANSPARENT),
        };

        let blend_img = match inputs.get(Self::BLEND_IN) {
            Some(NodeOutputData::Texture(img)) => img.clone(),
            _ => RgbaImage::new(ctx.width, ctx.height, RgbaColor::TRANSPARENT),
        };

        let opacity = self.opacity.evaluate_at(ctx.time).clamp(0.0, 1.0) as f32;
        let w = ctx.width;
        let h = ctx.height;
        let mut out_img = RgbaImage::new(w, h, RgbaColor::TRANSPARENT);

        for y in 0..h {
            for x in 0..w {
                let base_col = base_img.get_pixel(x, y);
                let blend_col = blend_img.get_pixel(x, y);

                let src_a = blend_col.a * opacity;
                let dst_a = base_col.a;

                let blended_r = Self::blend_channel(self.blend_mode, base_col.r, blend_col.r);
                let blended_g = Self::blend_channel(self.blend_mode, base_col.g, blend_col.g);
                let blended_b = Self::blend_channel(self.blend_mode, base_col.b, blend_col.b);

                // Porter-Duff Over compositing with blend mode result
                let out_a = src_a + dst_a * (1.0 - src_a);
                let (out_r, out_g, out_b) = if out_a > 1e-6 {
                    let r = (blended_r * src_a + base_col.r * dst_a * (1.0 - src_a)) / out_a;
                    let g = (blended_g * src_a + base_col.g * dst_a * (1.0 - src_a)) / out_a;
                    let b = (blended_b * src_a + base_col.b * dst_a * (1.0 - src_a)) / out_a;
                    (r, g, b)
                } else {
                    (0.0, 0.0, 0.0)
                };

                out_img.set_pixel(x, y, RgbaColor::new(out_r, out_g, out_b, out_a));
            }
        }

        let mut outs = HashMap::new();
        outs.insert(Self::TEX_OUT.into(), NodeOutputData::Texture(out_img));
        Ok(outs)
    }
}
