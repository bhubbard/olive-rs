use std::collections::HashMap;
use uuid::Uuid;

use crate::buffer::AudioBuffer;
use crate::error::Result;
use crate::graph::node::{EvaluationContext, Node, NodeOutputData};
use crate::graph::pin::{Pin, PinType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathOperation {
    Add,
    Multiply,
    Mix,
}

#[derive(Debug)]
pub struct MathNode {
    pub id: Uuid,
    pub name: String,
    pub operation: MathOperation,
}

impl MathNode {
    pub const PARAM_A_IN: &'static str = "param_a_in";
    pub const PARAM_B_IN: &'static str = "param_b_in";
    pub const SMP_OUT: &'static str = "smp_out";

    pub fn new(name: impl Into<String>, operation: MathOperation) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            operation,
        }
    }
}

impl Node for MathNode {
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
            Pin::input(Self::PARAM_A_IN, "Param A In", PinType::Samples),
            Pin::input(Self::PARAM_B_IN, "Param B In", PinType::Samples),
        ]
    }

    fn outputs(&self) -> Vec<Pin> {
        vec![Pin::output(Self::SMP_OUT, "Samples Out", PinType::Samples)]
    }

    fn evaluate(
        &self,
        ctx: &EvaluationContext,
        inputs: &HashMap<String, NodeOutputData>,
    ) -> Result<HashMap<String, NodeOutputData>> {
        let default_audio = AudioBuffer::new(2, ctx.sample_rate, (ctx.sample_rate / 30) as usize);

        let a = match inputs.get(Self::PARAM_A_IN) {
            Some(NodeOutputData::Samples(buf)) => buf.clone(),
            _ => default_audio.clone(),
        };

        let b = match inputs.get(Self::PARAM_B_IN) {
            Some(NodeOutputData::Samples(buf)) => buf.clone(),
            _ => default_audio,
        };

        let mut out = a;
        out.mix(&b, 1.0);

        let mut outs = HashMap::new();
        outs.insert(Self::SMP_OUT.into(), NodeOutputData::Samples(out));
        Ok(outs)
    }
}
