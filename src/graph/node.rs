use std::collections::HashMap;
use std::fmt::Debug;
use num_rational::Rational64;
use uuid::Uuid;

use crate::buffer::{AudioBuffer, RgbaColor, RgbaImage};
use crate::error::Result;
use crate::graph::pin::Pin;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeOutputData {
    Texture(RgbaImage),
    Samples(AudioBuffer),
    Float(f64),
    Color(RgbaColor),
    Empty,
}

#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub time: Rational64,
    pub width: u32,
    pub height: u32,
    pub sample_rate: u32,
}

pub trait Node: Debug + Send + Sync {
    fn id(&self) -> Uuid;
    fn name(&self) -> &str;
    fn category(&self) -> &str;
    fn inputs(&self) -> Vec<Pin>;
    fn outputs(&self) -> Vec<Pin>;

    fn evaluate(
        &self,
        ctx: &EvaluationContext,
        inputs: &HashMap<String, NodeOutputData>,
    ) -> Result<HashMap<String, NodeOutputData>>;
}
