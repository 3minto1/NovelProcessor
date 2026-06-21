use crate::domain::{ModelOutput, ModelProfile};
use crate::model_support::{format_request_error, ModelResponseError};
use crate::rate_limit::{RateLimitCoordinator, RateLimitScope};
use reqwest::Client;
use serde_json::json;
use std::time::Instant;

pub(crate) fn is_deepseek_profile(profile: &ModelProfile, base_url: &str, model: &str) -> bool {
    let provider = profile.provider.to_ascii_lowercase();
    let base = base_url.to_ascii_lowercase();
    let model = model.to_ascii_lowercase();
    provider.contains("deepseek") || base.contains("deepseek") || model.contains("deepseek")
}

pub(crate) fn is_gemini_profile(profile: &ModelProfile) -> bool {
    let provider = profile.provider.to_ascii_lowercase();
    let base = profile.base_url.to_ascii_lowercase();
    provider.contains("gemini") || base.contains("generativelanguage")
}

pub(crate) fn apply_openai_compatible_output_limit(
    payload: &mut serde_json::Value,
    profile: &ModelProfile,
    base_url: &str,
    model: &str,
    prefer_json_output: bool,
    output_limit_override: Option<usize>,
) -> bool {
    let output_limit = output_limit_override.unwrap_or_else(|| {
        if prefer_json_output { 16_384 } else { 65_536 }
    });

    if is_deepseek_profile(profile, base_url, model) {
        payload["max_tokens"] = json!(output_limit);
        return true;
    }

    payload["max_completion_tokens"] = json!(output_limit);
    true
}

pub(crate) fn openai_compatible_json_response_format(
    profile: &ModelProfile,
    base_url: &str,
    model: &str,
) -> Option<serde_json::Value> {
    if is_deepseek_profile(profile, base_url, model) {
        return Some(json!({ "type": "json_object" }));
    }
    None
}

pub(crate) fn apply_gemini_json_response_format(
    payload: &mut serde_json::Value,
    prefer_json_output: bool,
) -> bool {
    if !prefer_json_output {
        return false;
    }
    payload["generationConfig"]["responseMimeType"] = json!("application/json");
    true
}

pub(crate) fn apply_openai_compatible_thinking_control(
    payload: &mut serde_json::Value,
    profile: &ModelProfile,
    base_url: &str,
    model: &str,
) -> bool {
    match profile.thinking_mode.as_str() {
        "off" => {
            if is_deepseek_profile(profile, base_url, model) {
                payload["thinking"] = json!({ "type": "disabled" });
                return true;
            }
            false
        }
        "on" => {
            if is_deepseek_profile(profile, base_url, model) {
                payload["thinking"] = json!({ "type": "enabled" });
                return true;
            }
            false
        }
        _ => false,
    }
}

pub(crate) fn apply_gemini_thinking_control(
    payload: &mut serde_json::Value,
    profile: &ModelProfile,
) -> bool {
    let mode = profile.thinking_mode.as_str();
    if mode == "auto" {
        return false;
    }
    let thinking_config = if mode == "off" {
        json!({ "thinkingBudget": 0 })
    } else {
        json!({ "thinkingBudget": -1 })
    };
    payload["generationConfig"]["thinkingConfig"] = thinking_config;
    true
}

pub(crate) fn normalize_model_name(base_url: &str, model: &str) -> String {
    let trimmed = model.trim();
    if base_url.to_ascii_lowercase().contains("api.deepseek.com") {
        trimmed.to_ascii_lowercase()
    } else {
        trimmed.to_string()
    }
}

pub(crate) async fn generate_text(
    client: &Client,
    rate_limiter: Option<RateLimitCoordinator>,
    profile: &ModelProfile,
    api_key: &str,
    system: &str,
    user: &str,
    prefer_json_output: bool,
) -> Result<ModelOutput, String> {
    let scope = RateLimitScope::for_profile(profile);
    let started = Instant::now();
    let input_chars = system.chars().count() + user.chars().count();

    let result = if is_gemini_profile(profile) {
        super::gemini::generate_gemini(client, profile, api_key, system, user, prefer_json_output)
            .await
    } else {
        super::openai::generate_openai_compatible(
            client, profile, api_key, system, user, prefer_json_output, None,
        )
        .await
    };
    
    result
        .map(|mut output| {
            output.input_chars = input_chars;
            output.output_chars = output.text.chars().count();
            output.elapsed_ms = started.elapsed().as_millis();
            output
        })
        .map_err(|e| e.to_string())
}
