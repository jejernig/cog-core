//! Cog Engine — AI Coworker Middleware for Games
//!
//! A cognitive architecture for cooperative AI gameplay.
//! Gives NPCs goals, memory, relationships, opinions, and the ability to learn.

pub mod types;
pub mod agent;
pub mod memory;
pub mod personality;
pub mod relations;
pub mod world;
pub mod llm;
pub mod decision;

pub use types::*;
