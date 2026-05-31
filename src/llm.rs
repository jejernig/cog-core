//! LLM Client — provider abstraction for LLM inference.
//!
//! v0.5: hardcoded OpenAI-compatible provider.
//! v1.0: full model router with multi-provider support.

use crate::types::*;

/// Response from an LLM call.
pub struct LlmResponse {
    pub text: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub latency_ms: u64,
    pub model: String,
}

/// LLM client abstraction.
pub struct LlmClient {
    api_key: String,
    model: String,
    base_url: String,
}

impl LlmClient {
    /// Create a new LLM client.
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    /// Send a completion request.
    /// v0.5: synchronous stub. v1.0: async with full router.
    pub fn complete(&self, system_prompt: &str, user_prompt: &str) -> CogResult<LlmResponse> {
        // v0.5: placeholder — returns canned response for testing
        // Real implementation calls the OpenAI-compatible API
        let text = format!(
            "[{}] I've observed: {}",
            self.model,
            &user_prompt[..user_prompt.len().min(60)]
        );

        Ok(LlmResponse {
            text,
            input_tokens: (system_prompt.len() + user_prompt.len()) as u64 / 4,
            output_tokens: 50,
            latency_ms: 500,
            model: self.model.clone(),
        })
    }

    /// Get the model name.
    pub fn model(&self) -> &str {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_response() {
        let client = LlmClient::new("test_key", "gpt-4.1-nano");
        let resp = client.complete("You are a librarian.", "Where should I put The Hobbit?").unwrap();
        assert!(!resp.text.is_empty());
        assert!(resp.input_tokens > 0);
    }
}
