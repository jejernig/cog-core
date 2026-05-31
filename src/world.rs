//! World-State Layer
//!
//! Immutable event log that all agents can query.
//! Scoped access — agents only see events they've observed.

use crate::types::*;
use std::collections::HashMap;
use std::sync::Mutex;

/// A world event with metadata.
#[derive(Debug, Clone)]
pub struct WorldEvent {
    pub id: u64,
    pub timestamp: f64,
    pub content: String,
    pub category: ObservationCategory,
    pub position: Option<(f32, f32, f32)>, // 3D position for proximity queries
}

/// The world-state manager.
pub struct WorldState {
    events: Mutex<Vec<WorldEvent>>,
    /// Which events each agent has observed (by event ID).
    agent_visibility: Mutex<HashMap<AgentId, Vec<u64>>>,
    next_id: Mutex<u64>,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            agent_visibility: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }

    /// Log a new world event.
    pub fn log(&self, content: &str, category: ObservationCategory) -> u64 {
        let mut next = self.next_id.lock().unwrap();
        let id = *next;
        *next += 1;

        let event = WorldEvent {
            id,
            timestamp: std::time::UNIX_EPOCH.elapsed().unwrap_or_default().as_secs_f64(),
            content: content.to_string(),
            category,
            position: None,
        };

        self.events.lock().unwrap().push(event);
        id
    }

    /// Mark an event as observed by an agent.
    pub fn mark_observed(&self, agent_id: &str, event_id: u64) {
        let mut vis = self.agent_visibility.lock().unwrap();
        vis.entry(agent_id.to_string()).or_default().push(event_id);
    }

    /// Get events visible to an agent since a timestamp.
    pub fn query_visible(&self, agent_id: &str, since: f64) -> Vec<WorldEvent> {
        let vis = self.agent_visibility.lock().unwrap();
        let observed = vis.get(agent_id).cloned().unwrap_or_default();
        let events = self.events.lock().unwrap();

        events.iter()
            .filter(|e| e.timestamp >= since && observed.contains(&e.id))
            .cloned()
            .collect()
    }

    /// Get all events since a timestamp (for the player/debug).
    pub fn query_all(&self, since: f64) -> Vec<WorldEvent> {
        self.events.lock().unwrap()
            .iter()
            .filter(|e| e.timestamp >= since)
            .cloned()
            .collect()
    }

    /// Total event count.
    pub fn event_count(&self) -> usize {
        self.events.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_and_query() {
        let world = WorldState::new();
        let id = world.log("Book placed on shelf", ObservationCategory::PlayerAction);
        world.mark_observed("npc1", id);

        let visible = world.query_visible("npc1", 0.0);
        assert_eq!(visible.len(), 1);
        assert!(visible[0].content.contains("Book placed"));
    }

    #[test]
    fn test_scoped_visibility() {
        let world = WorldState::new();
        let id1 = world.log("Event A", ObservationCategory::PlayerAction);
        let id2 = world.log("Event B", ObservationCategory::PlayerAction);
        world.mark_observed("npc1", id1);
        // npc1 did NOT observe event B

        let visible = world.query_visible("npc1", 0.0);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].id, id1);
    }
}
