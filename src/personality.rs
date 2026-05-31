//! Personality Engine
//!
//! Builds system prompts from personality profiles.
//! The profile defines traits, philosophy, and voice — the engine assembles them.

use crate::types::*;

/// Build a system prompt for an agent from its personality profile.
pub fn build_system_prompt(profile: &PersonalityProfile) -> String {
    let traits_str = profile.traits.iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<_>>()
        .join(", ");

    let philosophy_str = profile.categorization_philosophy.iter()
        .map(|cp| format!("  {}. {} (priority {})", 
            cp.category, cp.priority, cp.priority))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "You are {name}, the {role}.\n\n\
         Traits: {traits}\n\n\
         Organizational philosophy (ranked):\n{philosophy}\n\n\
         Voice: {voice}\n\n\
         Stay in character. Be concise (2-4 sentences).\n\
         Constraints: {constraints}",
        name = profile.name,
        role = profile.role,
        traits = traits_str,
        philosophy = philosophy_str,
        voice = profile.voice_template,
        constraints = profile.constraints,
    )
}

/// Build the user prompt for a categorization request.
pub fn build_categorization_prompt(item: &str, context: &str) -> String {
    format!(
        "The player just placed '{}' on a shelf.\nContext: {}\n\
         Where should this book go? Respond with your reasoning and a suggested shelf label.",
        item, context
    )
}

/// Build the user prompt for a debate trigger.
pub fn build_debate_prompt(topic: &str, other_npc_opinion: &str) -> String {
    format!(
        "Another NPC says: \"{}\"\n\n\
         What do you say in response? Engage with their argument directly.",
        other_npc_opinion
    )
}

/// Build the user prompt for observing player behavior.
pub fn build_observation_prompt(observation: &str) -> String {
    format!(
        "You observe: {}\n\n\
         What do you make of this? Share your thoughts about the player's organizational style.",
        observation
    )
}

/// Create the three default NPC profiles for the Library Simulator.
pub fn default_profiles() -> Vec<PersonalityProfile> {
    vec![
        PersonalityProfile {
            name: "Archibald".into(),
            role: "Meticulous Archivist".into(),
            traits: [
                ("meticulous".into(), 0.95),
                ("pragmatic".into(), 0.1),
                ("creative".into(), 0.1),
                ("stubborn".into(), 0.7),
                ("humorous".into(), 0.1),
                ("helpful".into(), 0.8),
                ("contrarian".into(), 0.2),
            ].into_iter().collect(),
            categorization_philosophy: vec![
                CategoryPriority { category: "dewey_decimal".into(), priority: 1 },
                CategoryPriority { category: "alphabetical_author".into(), priority: 2 },
            ],
            voice_template: "Formal, precise, uses full sentences. Sometimes lectures. Says 'one' instead of 'you'.".into(),
            constraints: "Never reorganize without explicit request. Dewey Decimal is non-negotiable.".into(),
        },
        PersonalityProfile {
            name: "Maggie".into(),
            role: "Pragmatic Organizer".into(),
            traits: [
                ("meticulous".into(), 0.2),
                ("pragmatic".into(), 0.95),
                ("creative".into(), 0.4),
                ("stubborn".into(), 0.3),
                ("humorous".into(), 0.7),
                ("helpful".into(), 0.95),
                ("contrarian".into(), 0.4),
            ].into_iter().collect(),
            categorization_philosophy: vec![
                CategoryPriority { category: "usability".into(), priority: 1 },
                CategoryPriority { category: "proximity".into(), priority: 2 },
                CategoryPriority { category: "genre".into(), priority: 3 },
            ],
            voice_template: "Casual, warm, uses contractions. Sometimes dramatic. Says 'honey' when making a point.".into(),
            constraints: "Books go where people find them. Rules are suggestions.".into(),
        },
        PersonalityProfile {
            name: "Finn".into(),
            role: "Lore-Focused Taxonomist".into(),
            traits: [
                ("meticulous".into(), 0.1),
                ("pragmatic".into(), 0.1),
                ("creative".into(), 0.95),
                ("stubborn".into(), 0.5),
                ("humorous".into(), 0.4),
                ("helpful".into(), 0.7),
                ("contrarian".into(), 0.6),
            ].into_iter().collect(),
            categorization_philosophy: vec![
                CategoryPriority { category: "fictional_universe".into(), priority: 1 },
                CategoryPriority { category: "thematic".into(), priority: 2 },
                CategoryPriority { category: "author_connections".into(), priority: 3 },
            ],
            voice_template: "Enthusiastic, nerdy, uses exclamation points. References obscure literary connections. Says 'ACTUALLY' a lot.".into(),
            constraints: "Books belong with their fictional families. Genres are mere suggestions.".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_system_prompt() {
        let profiles = default_profiles();
        let prompt = build_system_prompt(&profiles[0]);
        assert!(prompt.contains("Archibald"));
        assert!(prompt.contains("Meticulous Archivist"));
        assert!(prompt.contains("Dewey Decimal"));
    }

    #[test]
    fn test_three_profiles_are_distinct() {
        let profiles = default_profiles();
        let p0 = build_system_prompt(&profiles[0]);
        let p1 = build_system_prompt(&profiles[1]);
        let p2 = build_system_prompt(&profiles[2]);
        assert_ne!(p0, p1);
        assert_ne!(p1, p2);
        assert_ne!(p0, p2);
    }
}
