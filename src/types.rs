//! Cog Engine — Core Types
//!
//! Fundamental data types shared across all engine subsystems.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for an agent (NPC).
pub type AgentId = String;

/// Observation category for memory tiering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObservationCategory {
    PlayerAction,
    NpcComment,
    WorldEvent,
    NpcThought,
}

/// An observation pushed into an agent's memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub timestamp: f64,
    pub content: String,
    pub category: ObservationCategory,
    pub metadata: HashMap<String, String>,
}

impl Observation {
    pub fn new(content: impl Into<String>, category: ObservationCategory) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64(),
            content: content.into(),
            category,
            metadata: HashMap::new(),
        }
    }
}

/// Output from an agent decision cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    pub agent_id: AgentId,
    pub agent_name: String,
    pub action: AgentAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentAction {
    /// NPC speaks or comments
    Speak {
        text: String,
        emotion: String,       // neutral, happy, annoyed, confused, excited
        target: Option<AgentId>, // None = directed at player
        intensity: u8,         // 1-5
    },
    /// NPC suggests a categorization
    Suggest {
        item: String,
        shelf: String,
        reasoning: String,
    },
    /// NPC disagrees with another NPC
    Debate {
        target_agent: AgentId,
        argument: String,
    },
    /// NPC takes no action (thinking or idle)
    Idle {
        reason: String,
    },
}

/// A personality profile defining an agent's traits and behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityProfile {
    pub name: String,
    pub role: String,
    pub traits: HashMap<String, f32>,
    pub categorization_philosophy: Vec<CategoryPriority>,
    pub voice_template: String,
    pub constraints: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryPriority {
    pub category: String,
    pub priority: u8,
}

/// Relationship weights between two agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipWeights {
    pub trust: f32,
    pub respect: f32,
    pub affection: f32,
    pub annoyance: f32,
}

impl Default for RelationshipWeights {
    fn default() -> Self {
        Self { trust: 0.5, respect: 0.5, affection: 0.3, annoyance: 0.1 }
    }
}

/// A memory entry stored in episodic memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: i64,
    pub content: String,
    pub similarity: f32,
    pub importance: f32,
    pub timestamp: f64,
}

/// Engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CogConfig {
    pub api_key: String,
    pub model: String,
    pub tier: ModelTier,
    pub max_concurrent_agents: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelTier {
    Budget,
    Standard,
    Premium,
}

/// Errors that can occur in the Cog engine.
#[derive(Debug, thiserror::Error)]
pub enum CogError {
    #[error("Agent {0} not found")]
    AgentNotFound(AgentId),
    #[error("LLM error: {0}")]
    LlmError(String),
    #[error("Memory error: {0}")]
    MemoryError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type CogResult<T> = Result<T, CogError>;
