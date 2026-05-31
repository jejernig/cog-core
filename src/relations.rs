//! Relationship Graph
//!
//! Weighted directed graph tracking NPC↔NPC and NPC↔Player relationships.
//! Supports damped propagation — when A's opinion of B changes, A's allies shift too.

use crate::types::*;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;

pub struct RelationshipGraph {
    graph: DiGraph<AgentId, RelationshipWeights>,
    nodes: HashMap<AgentId, NodeIndex>,
}

impl RelationshipGraph {
    pub fn new() -> Self {
        Self { graph: DiGraph::new(), nodes: HashMap::new() }
    }

    /// Register an agent in the graph.
    pub fn register(&mut self, agent_id: &str) {
        if !self.nodes.contains_key(agent_id) {
            let idx = self.graph.add_node(agent_id.to_string());
            self.nodes.insert(agent_id.to_string(), idx);
        }
    }

    /// Set or update relationship from `from` to `to`.
    pub fn set_relationship(&mut self, from: &str, to: &str, weights: RelationshipWeights) -> CogResult<()> {
        let from_idx = *self.nodes.get(from).ok_or_else(|| CogError::AgentNotFound(from.into()))?;
        let to_idx = *self.nodes.get(to).ok_or_else(|| CogError::AgentNotFound(to.into()))?;

        // Remove existing edge if any, then add new
        if let Some(edge) = self.graph.find_edge(from_idx, to_idx) {
            self.graph.remove_edge(edge);
        }
        self.graph.add_edge(from_idx, to_idx, weights);
        Ok(())
    }

    /// Update one dimension of a relationship.
    pub fn update(&mut self, from: &str, to: &str, dimension: &str, delta: f32) -> CogResult<()> {
        let from_idx = *self.nodes.get(from).ok_or_else(|| CogError::AgentNotFound(from.into()))?;
        let to_idx = *self.nodes.get(to).ok_or_else(|| CogError::AgentNotFound(to.into()))?;

        let edge = self.graph.find_edge(from_idx, to_idx)
            .unwrap_or_else(|| {
                self.graph.add_edge(from_idx, to_idx, RelationshipWeights::default());
                self.graph.find_edge(from_idx, to_idx).unwrap()
            });

        let mut weights = self.graph[edge].clone();
        match dimension {
            "trust" => weights.trust = (weights.trust + delta).clamp(0.0, 1.0),
            "respect" => weights.respect = (weights.respect + delta).clamp(0.0, 1.0),
            "affection" => weights.affection = (weights.affection + delta).clamp(0.0, 1.0),
            "annoyance" => weights.annoyance = (weights.annoyance + delta).clamp(0.0, 1.0),
            _ => return Err(CogError::Internal(format!("Unknown dimension: {}", dimension))),
        }
        self.graph[edge] = weights;
        Ok(())
    }

    /// Propagate a relationship change to allies of `from`.
    /// Allies (those `from` trusts) shift their opinion of `to` by delta * decay * trust(from→ally).
    pub fn propagate(&mut self, from: &str, to: &str, dimension: &str, delta: f32, decay: f32) -> CogResult<()> {
        // First, update the direct relationship
        self.update(from, to, dimension, delta)?;

        // Find allies — agents that `from` trusts (trust > 0.4)
        let from_idx = *self.nodes.get(from).ok_or_else(|| CogError::AgentNotFound(from.into()))?;
        let mut allies: Vec<(AgentId, f32)> = Vec::new();

        for edge in self.graph.edges(from_idx) {
            let ally_id = self.graph[edge.target()].clone();
            let trust = edge.weight().trust;
            if trust > 0.4 && ally_id != to {
                allies.push((ally_id, trust));
            }
        }

        // Propagate damped change to allies
        for (ally_id, trust) in allies {
            let ally_delta = delta * trust * decay;
            if ally_delta.abs() > 0.01 {
                self.update(&ally_id, to, dimension, ally_delta)?;
            }
        }

        Ok(())
    }

    /// Get agents trusted by `agent_id` above a threshold.
    pub fn get_trusted_peers(&self, agent_id: &str, threshold: f32) -> CogResult<Vec<(AgentId, f32)>> {
        let idx = *self.nodes.get(agent_id).ok_or_else(|| CogError::AgentNotFound(agent_id.into()))?;
        let mut peers: Vec<(AgentId, f32)> = Vec::new();

        for edge in self.graph.edges(idx) {
            if edge.weight().trust >= threshold {
                peers.push((self.graph[edge.target()].clone(), edge.weight().trust));
            }
        }

        peers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(peers)
    }

    /// Get the relationship from `from` to `to`.
    pub fn get_relationship(&self, from: &str, to: &str) -> CogResult<RelationshipWeights> {
        let from_idx = *self.nodes.get(from).ok_or_else(|| CogError::AgentNotFound(from.into()))?;
        let to_idx = *self.nodes.get(to).ok_or_else(|| CogError::AgentNotFound(to.into()))?;

        if let Some(edge) = self.graph.find_edge(from_idx, to_idx) {
            Ok(self.graph[edge].clone())
        } else {
            Ok(RelationshipWeights::default())
        }
    }

    /// Export the full relationship map as JSON.
    pub fn export_json(&self) -> CogResult<String> {
        let mut map: HashMap<String, HashMap<String, RelationshipWeights>> = HashMap::new();

        for edge in self.graph.edge_references() {
            let from = &self.graph[edge.source()];
            let to = &self.graph[edge.target()];
            map.entry(from.clone())
                .or_default()
                .insert(to.clone(), edge.weight().clone());
        }

        serde_json::to_string_pretty(&map).map_err(CogError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_graph() -> RelationshipGraph {
        let mut g = RelationshipGraph::new();
        for id in &["a", "b", "c"] {
            g.register(id);
        }
        g
    }

    #[test]
    fn test_basic_relationship() {
        let mut g = setup_graph();
        g.set_relationship("a", "b", RelationshipWeights {
            trust: 0.9, respect: 0.7, affection: 0.5, annoyance: 0.1
        }).unwrap();

        let r = g.get_relationship("a", "b").unwrap();
        assert_eq!(r.trust, 0.9);
    }

    #[test]
    fn test_propagation() {
        let mut g = setup_graph();
        // a trusts b (0.9) and c (0.6)
        g.set_relationship("a", "b", RelationshipWeights { trust: 0.9, ..Default::default() }).unwrap();
        g.set_relationship("a", "c", RelationshipWeights { trust: 0.6, ..Default::default() }).unwrap();

        // When a's trust in b drops, allies shift
        g.propagate("a", "b", "trust", -0.5, 0.5).unwrap();

        // a→b should have dropped
        let a_to_b = g.get_relationship("a", "b").unwrap();
        assert!(a_to_b.trust < 0.5); // was 0.9, now 0.4

        // c (ally with trust 0.6) should also shift
        // delta = -0.5 * 0.6 * 0.5 = -0.15
        let c_to_b = g.get_relationship("c", "b").unwrap();
        assert!(c_to_b.trust < 0.5); // was 0.5, now ~0.35
    }
}
