// Preset configurations for common API providers

use crate::utils::{CcmError, Result};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Preset {
    pub name: String,
    pub description: String,
    pub default_fields: HashMap<String, String>,
    pub env_mapping: HashMap<String, String>,
    pub required_fields: Vec<String>,
}

/// Get preset by name
pub fn get_preset(name: &str) -> Result<Preset> {
    match name.to_lowercase().as_str() {
        "claude" => Ok(claude_preset()),
        "openai" => Ok(openai_preset()),
        "gemini" => Ok(gemini_preset()),
        "github" => Ok(github_preset()),
        "aws" => Ok(aws_preset()),
        _ => Err(CcmError::InvalidArgument(format!(
            "Unknown preset: {}. Available: claude, openai, gemini, github, aws",
            name
        ))),
    }
}

/// List all available presets
pub fn list_presets() -> Vec<Preset> {
    vec![
        claude_preset(),
        openai_preset(),
        gemini_preset(),
        github_preset(),
        aws_preset(),
    ]
}

fn claude_preset() -> Preset {
    let mut default_fields = HashMap::new();
    default_fields.insert("url".to_string(), "https://api.anthropic.com".to_string());

    let mut env_mapping = HashMap::new();
    env_mapping.insert("token".to_string(), "ANTHROPIC_API_KEY".to_string());
    env_mapping.insert("url".to_string(), "ANTHROPIC_BASE_URL".to_string());
    env_mapping.insert("model".to_string(), "ANTHROPIC_MODEL".to_string());

    Preset {
        name: "claude".to_string(),
        description: "Anthropic Claude API".to_string(),
        default_fields,
        env_mapping,
        required_fields: vec!["token".to_string()],
    }
}

fn openai_preset() -> Preset {
    let mut default_fields = HashMap::new();
    default_fields.insert("url".to_string(), "https://api.openai.com/v1".to_string());

    let mut env_mapping = HashMap::new();
    env_mapping.insert("token".to_string(), "OPENAI_API_KEY".to_string());
    env_mapping.insert("url".to_string(), "OPENAI_BASE_URL".to_string());
    env_mapping.insert("model".to_string(), "OPENAI_MODEL".to_string());

    Preset {
        name: "openai".to_string(),
        description: "OpenAI API".to_string(),
        default_fields,
        env_mapping,
        required_fields: vec!["token".to_string()],
    }
}

fn gemini_preset() -> Preset {
    let mut default_fields = HashMap::new();
    default_fields.insert(
        "url".to_string(),
        "https://generativelanguage.googleapis.com".to_string(),
    );

    let mut env_mapping = HashMap::new();
    env_mapping.insert("token".to_string(), "GEMINI_API_KEY".to_string());
    env_mapping.insert("url".to_string(), "GEMINI_BASE_URL".to_string());
    env_mapping.insert("model".to_string(), "GEMINI_MODEL".to_string());

    Preset {
        name: "gemini".to_string(),
        description: "Google Gemini API".to_string(),
        default_fields,
        env_mapping,
        required_fields: vec!["token".to_string()],
    }
}

fn github_preset() -> Preset {
    let mut default_fields = HashMap::new();
    default_fields.insert("url".to_string(), "https://api.github.com".to_string());

    let mut env_mapping = HashMap::new();
    env_mapping.insert("token".to_string(), "GITHUB_TOKEN".to_string());
    env_mapping.insert("url".to_string(), "GITHUB_API_URL".to_string());

    Preset {
        name: "github".to_string(),
        description: "GitHub API".to_string(),
        default_fields,
        env_mapping,
        required_fields: vec!["token".to_string()],
    }
}

fn aws_preset() -> Preset {
    let mut default_fields = HashMap::new();
    default_fields.insert("region".to_string(), "us-east-1".to_string());

    let mut env_mapping = HashMap::new();
    env_mapping.insert("access_key".to_string(), "AWS_ACCESS_KEY_ID".to_string());
    env_mapping.insert("secret_key".to_string(), "AWS_SECRET_ACCESS_KEY".to_string());
    env_mapping.insert("region".to_string(), "AWS_REGION".to_string());
    env_mapping.insert("session_token".to_string(), "AWS_SESSION_TOKEN".to_string());

    Preset {
        name: "aws".to_string(),
        description: "AWS API".to_string(),
        default_fields,
        env_mapping,
        required_fields: vec!["access_key".to_string(), "secret_key".to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_preset() {
        let preset = get_preset("claude").unwrap();
        assert_eq!(preset.name, "claude");
        assert!(preset.env_mapping.contains_key("token"));
    }

    #[test]
    fn test_list_presets() {
        let presets = list_presets();
        assert_eq!(presets.len(), 5);
    }
}
