//! Agent Runtime — lifecycle, scheduling, concurrent execution.
//!
//! Spawns, suspends, resumes, and terminates AI agents.
//! Agents run concurrently via tokio tasks with priority scheduling.

use crate::types::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// The core agent runtime. Manages all agent lifecycles.
pub struct AgentRuntime {
    config: CogConfig,
    agents: RwLock<HashMap<AgentId, Arc<Agent>>>,
    next_id: Mutex<u64>,
}

/// An active AI agent (NPC).
pub struct Agent {
    pub id: AgentId,
    pub profile: PersonalityProfile,
    pub state: RwLock<AgentState>,
}

#[derive(Debug, Clone)]
pub struct AgentState {
    pub status: AgentStatus,
    pub token_budget_remaining: u64,
    pub total_tokens_used: u64,
    pub last_tick: Option<f64>,
    pub suspended_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Idle,
    Thinking,
    Speaking,
    Suspended,
    Terminated,
}

impl AgentRuntime {
    /// Create a new agent runtime with the given config.
    pub fn new(config: CogConfig) -> Self {
        Self {
            config,
            agents: RwLock::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }

    /// Spawn a new agent with the given personality profile.
    pub async fn spawn(&self, profile: PersonalityProfile) -> CogResult<AgentId> {
        let mut next = self.next_id.lock().await;
        let id = format!("agent_{:04}", *next);
        *next += 1;
        drop(next);

        let agent = Arc::new(Agent {
            id: id.clone(),
            profile,
            state: RwLock::new(AgentState {
                status: AgentStatus::Idle,
                token_budget_remaining: 100_000,
                total_tokens_used: 0,
                last_tick: None,
                suspended_reason: None,
            }),
        });

        self.agents.write().await.insert(id.clone(), agent);
        Ok(id)
    }

    /// Suspend an agent (pause its decision loop).
    pub async fn suspend(&self, agent_id: &AgentId, reason: &str) -> CogResult<()> {
        let agents = self.agents.read().await;
        let agent = agents.get(agent_id).ok_or_else(|| CogError::AgentNotFound(agent_id.clone()))?;
        let mut state = agent.state.write().await;
        state.status = AgentStatus::Suspended;
        state.suspended_reason = Some(reason.to_string());
        Ok(())
    }

    /// Resume a suspended agent.
    pub async fn resume(&self, agent_id: &AgentId) -> CogResult<()> {
        let agents = self.agents.read().await;
        let agent = agents.get(agent_id).ok_or_else(|| CogError::AgentNotFound(agent_id.clone()))?;
        let mut state = agent.state.write().await;
        state.status = AgentStatus::Idle;
        state.suspended_reason = None;
        Ok(())
    }

    /// Terminate and remove an agent.
    pub async fn terminate(&self, agent_id: &AgentId) -> CogResult<()> {
        let mut agents = self.agents.write().await;
        agents.remove(agent_id).ok_or_else(|| CogError::AgentNotFound(agent_id.clone()))?;
        Ok(())
    }

    /// Get an agent by ID.
    pub async fn get(&self, agent_id: &AgentId) -> CogResult<Arc<Agent>> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned().ok_or_else(|| CogError::AgentNotFound(agent_id.clone()))
    }

    /// List all active agent IDs.
    pub async fn list_agents(&self) -> Vec<AgentId> {
        self.agents.read().await.keys().cloned().collect()
    }

    /// Count of active agents.
    pub async fn agent_count(&self) -> usize {
        self.agents.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_profile(name: &str) -> PersonalityProfile {
        PersonalityProfile {
            name: name.to_string(),
            role: "Test".to_string(),
            traits: HashMap::new(),
            categorization_philosophy: vec![],
            voice_template: String::new(),
            constraints: String::new(),
        }
    }

    #[tokio::test]
    async fn test_spawn_terminate() {
        let runtime = AgentRuntime::new(CogConfig {
            api_key: "test".into(),
            model: "test".into(),
            tier: ModelTier::Budget,
            max_concurrent_agents: 10,
        });

        let id = runtime.spawn(test_profile("TestNPC")).await.unwrap();
        assert_eq!(runtime.agent_count().await, 1);

        let agent = runtime.get(&id).await.unwrap();
        assert_eq!(agent.state.read().await.status, AgentStatus::Idle);

        runtime.terminate(&id).await.unwrap();
        assert_eq!(runtime.agent_count().await, 0);
    }

    #[tokio::test]
    async fn test_suspend_resume() {
        let runtime = AgentRuntime::new(CogConfig {
            api_key: "test".into(), model: "test".into(),
            tier: ModelTier::Budget, max_concurrent_agents: 10,
        });

        let id = runtime.spawn(test_profile("Sleepy")).await.unwrap();
        runtime.suspend(&id, "nap time").await.unwrap();

        let agent = runtime.get(&id).await.unwrap();
        assert_eq!(agent.state.read().await.status, AgentStatus::Suspended);

        runtime.resume(&id).await.unwrap();
        assert_eq!(agent.state.read().await.status, AgentStatus::Idle);
    }
}
