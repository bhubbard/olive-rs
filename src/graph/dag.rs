use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{OliveError, Result};
use crate::graph::connection::Connection;
use crate::graph::node::{EvaluationContext, Node, NodeOutputData};
use crate::graph::pin::PinDirection;

#[derive(Debug, Clone)]
pub struct Graph {
    pub nodes: HashMap<Uuid, Arc<dyn Node>>,
    pub connections: Vec<Connection>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Arc<dyn Node>) {
        self.nodes.insert(node.id(), node);
    }

    pub fn remove_node(&mut self, id: &Uuid) {
        self.nodes.remove(id);
        self.connections
            .retain(|c| c.from_node != *id && c.to_node != *id);
    }

    pub fn connect(
        &mut self,
        from_node: Uuid,
        from_pin: impl Into<String>,
        to_node: Uuid,
        to_pin: impl Into<String>,
    ) -> Result<()> {
        let from_pin_s = from_pin.into();
        let to_pin_s = to_pin.into();

        let source = self
            .nodes
            .get(&from_node)
            .ok_or_else(|| OliveError::NodeNotFound(from_node.to_string()))?;
        let target = self
            .nodes
            .get(&to_node)
            .ok_or_else(|| OliveError::NodeNotFound(to_node.to_string()))?;

        let source_pin = source
            .outputs()
            .into_iter()
            .find(|p| p.id == from_pin_s && p.direction == PinDirection::Output)
            .ok_or_else(|| OliveError::PinNotFound(from_pin_s.clone()))?;

        let target_pin = target
            .inputs()
            .into_iter()
            .find(|p| p.id == to_pin_s && p.direction == PinDirection::Input)
            .ok_or_else(|| OliveError::PinNotFound(to_pin_s.clone()))?;

        if source_pin.pin_type != target_pin.pin_type && target_pin.pin_type != crate::graph::pin::PinType::Value {
            return Err(OliveError::IncompatibleTypes(
                format!("{:?}", source_pin.pin_type),
                format!("{:?}", target_pin.pin_type),
            ));
        }

        // Remove any existing connection to the same input pin
        self.connections
            .retain(|c| !(c.to_node == to_node && c.to_pin == to_pin_s));

        self.connections.push(Connection::new(
            from_node,
            from_pin_s,
            to_node,
            to_pin_s,
        ));

        // Verify that this didn't introduce a cycle
        if self.has_cycle() {
            self.connections.pop();
            return Err(OliveError::CycleDetected(format!(
                "Cycle detected connecting {} to {}",
                from_node, to_node
            )));
        }

        Ok(())
    }

    pub fn disconnect_input(&mut self, to_node: Uuid, to_pin: &str) {
        self.connections
            .retain(|c| !(c.to_node == to_node && c.to_pin == to_pin));
    }

    pub fn has_cycle(&self) -> bool {
        self.topological_sort().is_err()
    }

    pub fn topological_sort(&self) -> Result<Vec<Uuid>> {
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();
        let mut adj: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

        for id in self.nodes.keys() {
            in_degree.insert(*id, 0);
            adj.insert(*id, Vec::new());
        }

        for conn in &self.connections {
            if self.nodes.contains_key(&conn.from_node) && self.nodes.contains_key(&conn.to_node) {
                adj.entry(conn.from_node).or_default().push(conn.to_node);
                *in_degree.entry(conn.to_node).or_default() += 1;
            }
        }

        let mut queue: VecDeque<Uuid> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut sorted = Vec::new();

        while let Some(node_id) = queue.pop_front() {
            sorted.push(node_id);
            if let Some(neighbors) = adj.get(&node_id) {
                for &neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(&neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        if sorted.len() != self.nodes.len() {
            return Err(OliveError::CycleDetected(
                "Graph contains a directed cycle".into(),
            ));
        }

        Ok(sorted)
    }

    /// Evaluates all nodes in topological order and returns output maps for each node
    pub fn evaluate(
        &self,
        ctx: &EvaluationContext,
    ) -> Result<HashMap<Uuid, HashMap<String, NodeOutputData>>> {
        let order = self.topological_sort()?;
        let mut node_outputs: HashMap<Uuid, HashMap<String, NodeOutputData>> = HashMap::new();

        for node_id in order {
            let node = self.nodes.get(&node_id).unwrap();

            // Collect inputs for this node from incoming connections
            let mut inputs = HashMap::new();
            for conn in &self.connections {
                if conn.to_node == node_id {
                    if let Some(source_outs) = node_outputs.get(&conn.from_node) {
                        if let Some(val) = source_outs.get(&conn.from_pin) {
                            inputs.insert(conn.to_pin.clone(), val.clone());
                        }
                    }
                }
            }

            let outputs = node.evaluate(ctx, &inputs)?;
            node_outputs.insert(node_id, outputs);
        }

        Ok(node_outputs)
    }
}
