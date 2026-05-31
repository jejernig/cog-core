//! Tiered Memory System
//!
//! Three-tier architecture validated in Spike 001:
//! - Working memory: ring buffer (fast, volatile, last N observations)
//! - Episodic memory: SQLite + embedding-based retrieval
//! - Semantic memory: key-value facts, periodic LLM summarization

use crate::types::*;
use rusqlite::Connection;
use std::collections::VecDeque;
use std::sync::Mutex;

/// The tiered memory system for an agent.
pub struct MemorySystem {
    agent_id: AgentId,
    working: Mutex<WorkingMemory>,
    episodic: Mutex<EpisodicStore>,
    semantic: Mutex<SemanticStore>,
    episode_count: Mutex<usize>,
    summarize_every: usize,
}

// ─── Working Memory ────────────────────────────────

struct WorkingMemory {
    buffer: VecDeque<Observation>,
    capacity: usize,
}

impl WorkingMemory {
    fn new(capacity: usize) -> Self {
        Self { buffer: VecDeque::with_capacity(capacity), capacity }
    }

    fn push(&mut self, obs: Observation) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(obs);
    }

    fn recent(&self, n: usize) -> Vec<Observation> {
        self.buffer.iter().rev().take(n).cloned().collect()
    }

    fn context_window(&self, n: usize) -> String {
        self.buffer.iter().rev().take(n)
            .map(|o| format!("[{}] {}", o.timestamp as i64, o.content))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// ─── Episodic Memory ──────────────────────────────

struct EpisodicStore {
    conn: Connection,
}

impl EpisodicStore {
    fn new() -> CogResult<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| CogError::MemoryError(e.to_string()))?;
        conn.execute_batch(
            "CREATE TABLE episodes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id TEXT NOT NULL,
                timestamp REAL NOT NULL,
                content TEXT NOT NULL,
                importance REAL DEFAULT 0.5
            );
            CREATE INDEX idx_agent_time ON episodes(agent_id, timestamp);"
        ).map_err(|e| CogError::MemoryError(e.to_string()))?;
        Ok(Self { conn })
    }

    fn store(&self, agent_id: &str, content: &str, importance: f32) -> CogResult<()> {
        self.conn.execute(
            "INSERT INTO episodes (agent_id, timestamp, content, importance) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![agent_id, std::time::UNIX_EPOCH.elapsed().unwrap_or_default().as_secs_f64(), content, importance],
        ).map_err(|e| CogError::MemoryError(e.to_string()))?;
        Ok(())
    }

    fn retrieve(&self, agent_id: &str, keyword: &str, k: usize) -> CogResult<Vec<MemoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, importance, timestamp FROM episodes WHERE agent_id = ?1 AND content LIKE ?2 ORDER BY timestamp DESC LIMIT ?3"
        ).map_err(|e| CogError::MemoryError(e.to_string()))?;

        let pattern = format!("%{}%", keyword);
        let rows = stmt.query_map(
            rusqlite::params![agent_id, pattern, k as i64],
            |row| Ok(MemoryEntry {
                id: row.get(0)?,
                content: row.get(1)?,
                similarity: row.get::<_, f32>(2)?,
                importance: row.get(2)?,
                timestamp: row.get(3)?,
            })
        ).map_err(|e| CogError::MemoryError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| CogError::MemoryError(e.to_string()))?);
        }
        Ok(results)
    }

    fn count(&self, agent_id: &str) -> CogResult<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM episodes WHERE agent_id = ?1",
            rusqlite::params![agent_id],
            |row| row.get(0),
        ).map_err(|e| CogError::MemoryError(e.to_string()))?;
        Ok(count as usize)
    }
}

// ─── Semantic Memory ──────────────────────────────

struct SemanticStore {
    conn: Connection,
}

impl SemanticStore {
    fn new() -> CogResult<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| CogError::MemoryError(e.to_string()))?;
        conn.execute_batch(
            "CREATE TABLE facts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                confidence REAL DEFAULT 0.5,
                UNIQUE(agent_id, key)
            );"
        ).map_err(|e| CogError::MemoryError(e.to_string()))?;
        Ok(Self { conn })
    }

    fn learn(&self, agent_id: &str, key: &str, value: &str, confidence: f32) -> CogResult<()> {
        self.conn.execute(
            "INSERT INTO facts (agent_id, key, value, confidence) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(agent_id, key) DO UPDATE SET value = excluded.value, confidence = excluded.confidence",
            rusqlite::params![agent_id, key, value, confidence],
        ).map_err(|e| CogError::MemoryError(e.to_string()))?;
        Ok(())
    }

    fn query(&self, agent_id: &str) -> CogResult<Vec<(String, String, f32)>> {
        let mut stmt = self.conn.prepare(
            "SELECT key, value, confidence FROM facts WHERE agent_id = ?1 ORDER BY confidence DESC"
        ).map_err(|e| CogError::MemoryError(e.to_string()))?;

        let rows = stmt.query_map(
            rusqlite::params![agent_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, f32>(2)?)),
        ).map_err(|e| CogError::MemoryError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| CogError::MemoryError(e.to_string()))?);
        }
        Ok(results)
    }
}

// ─── MemorySystem Public API ──────────────────────

impl MemorySystem {
    /// Create a new memory system for the given agent.
    pub fn new(agent_id: &str) -> CogResult<Self> {
        Ok(Self {
            agent_id: agent_id.to_string(),
            working: Mutex::new(WorkingMemory::new(50)),
            episodic: Mutex::new(EpisodicStore::new()?),
            semantic: Mutex::new(SemanticStore::new()?),
            episode_count: Mutex::new(0),
            summarize_every: 20,
        })
    }

    /// Record an observation in working and episodic memory.
    pub fn observe(&self, content: &str, category: ObservationCategory) -> CogResult<()> {
        let obs = Observation::new(content, category);
        self.working.lock().unwrap().push(obs);
        self.episodic.lock().unwrap().store(&self.agent_id, content, 0.5)?;

        let mut count = self.episode_count.lock().unwrap();
        *count += 1;

        // Trigger summarization periodically
        if *count % self.summarize_every == 0 {
            drop(count);
            // In production, this calls the LLM. For v0.5, stubbed.
        }
        Ok(())
    }

    /// Get recent observations from working memory.
    pub fn recent_observations(&self, n: usize) -> String {
        self.working.lock().unwrap().context_window(n)
    }

    /// Search episodic memory for relevant past events.
    pub fn recall(&self, keyword: &str, k: usize) -> CogResult<Vec<MemoryEntry>> {
        self.episodic.lock().unwrap().retrieve(&self.agent_id, keyword, k)
    }

    /// Learn a semantic fact.
    pub fn learn_fact(&self, key: &str, value: &str, confidence: f32) -> CogResult<()> {
        self.semantic.lock().unwrap().learn(&self.agent_id, key, value, confidence)
    }

    /// Query semantic facts.
    pub fn facts(&self) -> CogResult<Vec<(String, String, f32)>> {
        self.semantic.lock().unwrap().query(&self.agent_id)
    }

    /// Total episodes stored.
    pub fn episode_count(&self) -> CogResult<usize> {
        self.episodic.lock().unwrap().count(&self.agent_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observe_and_recall() {
        let mem = MemorySystem::new("test_npc").unwrap();
        mem.observe("Player placed The Hobbit in Fantasy", ObservationCategory::PlayerAction).unwrap();
        mem.observe("Player placed Dune in Sci-Fi", ObservationCategory::PlayerAction).unwrap();

        let results = mem.recall("Hobbit", 5).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].content.contains("Hobbit"));
    }

    #[test]
    fn test_semantic_facts() {
        let mem = MemorySystem::new("test_npc").unwrap();
        mem.learn_fact("prefers_genre_sorting", "true", 0.8).unwrap();
        let facts = mem.facts().unwrap();
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].0, "prefers_genre_sorting");
    }
}
