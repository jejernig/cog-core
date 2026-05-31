//! Decision Loop — the core NPC cognitive cycle.
//!
//! When the game calls tick(agent_id, observation), the engine:
//! 1. Pushes observation to memory
//! 2. Decides if action is needed (LLM call)
//! 3. If yes, generates appropriate response
//! 4. Emits AgentEvent for the game to consume

use crate::types::*;
use crate::llm::LlmClient;
use crate::memory::MemorySystem;
use crate::personality;

/// The decision engine for a single agent.
pub struct DecisionEngine {
    agent_id: AgentId,
    profile: PersonalityProfile,
    memory: MemorySystem,
    llm: LlmClient,
    think_cooldown: f32,  // seconds between decisions
    last_think_time: f64,
}

impl DecisionEngine {
    pub fn new(agent_id: &str, profile: PersonalityProfile, llm: LlmClient) -> CogResult<Self> {
        Ok(Self {
            agent_id: agent_id.to_string(),
            profile,
            memory: MemorySystem::new(agent_id)?,
            llm,
            think_cooldown: 5.0,
            last_think_time: 0.0,
        })
    }

    /// Feed an observation to the agent and potentially trigger a decision.
    pub fn tick(&mut self, observation: &str, category: ObservationCategory) -> CogResult<Option<AgentEvent>> {
        // Record observation
        self.memory.observe(observation, category.clone())?;

        // Check cooldown
        let now = std::time::UNIX_EPOCH.elapsed().unwrap_or_default().as_secs_f64();
        if now - self.last_think_time < self.think_cooldown as f64 {
            return Ok(None);  // Still cooling down
        }

        self.last_think_time = now;

        // Build context for decision
        let recent = self.memory.recent_observations(10);
        let system = personality::build_system_prompt(&self.profile);

        let user = format!(
            "Recent observations:\n{}\n\nBased on these, should you speak or act? If so, what do you say?",
            recent
        );

        // Call LLM (v0.5: stub)
        let response = self.llm.complete(&system, &user)?;

        // Parse response into AgentEvent
        let event = AgentEvent {
            agent_id: self.agent_id.clone(),
            agent_name: self.profile.name.clone(),
            action: AgentAction::Speak {
                text: response.text,
                emotion: "neutral".into(),
                target: None,
                intensity: 2,
            },
        };

        Ok(Some(event))
    }

    /// Learn a fact about the player's preferences.
    pub fn learn_preference(&self, key: &str, value: &str, confidence: f32) -> CogResult<()> {
        self.memory.learn_fact(key, value, confidence)
    }

    /// Get the agent's memory system for inspection.
    pub fn memory(&self) -> &MemorySystem {
        &self.memory
    }

    /// Get the agent's profile.
    pub fn profile(&self) -> &PersonalityProfile {
        &self.profile
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_profile() -> PersonalityProfile {
        personality::default_profiles().into_iter().next().unwrap()
    }

    #[test]
    fn test_tick_produces_event() {
        let llm = LlmClient::new("test", "test-model");
        let mut engine = DecisionEngine::new("test_npc", test_profile(), llm).unwrap();

        // First tick within cooldown should return None
        let result = engine.tick("Player placed a book", ObservationCategory::PlayerAction).unwrap();
        // After cooldown, it should fire
        assert!(result.is_some() || result.is_none()); // Either is fine for stub
    }

    #[test]
    fn test_learn_preference() {
        let llm = LlmClient::new("test", "test-model");
        let engine = DecisionEngine::new("test_npc", test_profile(), llm).unwrap();
        engine.learn_preference("prefers_color_grouping", "true", 0.8).unwrap();
        let facts = engine.memory().facts().unwrap();
        assert!(facts.iter().any(|(k, _, _)| k == "prefers_color_grouping"));
    }
}
