use serde_json::Value;

#[derive(Debug)]
pub(crate) struct ModelResponseError {
    pub message: String,
    pub status: Option<u16>,
    pub retry_after: Option<u64>,
}

impl ModelResponseError {
    pub fn other(message: String) -> Self {
        Self {
            message,
            status: None,
            retry_after: None,
        }
    }

    pub fn provider(status: u16, message: String, retry_after: Option<u64>) -> Self {
        Self {
            message,
            status: Some(status),
            retry_after,
        }
    }

    pub fn permits_thinking_retry(&self) -> bool {
        self.status == Some(400) || self.status == Some(422)
    }
}

impl std::fmt::Display for ModelResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(status) = self.status {
            write!(f, "HTTP {}: {}", status, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for ModelResponseError {}

pub(crate) fn format_request_error(error: reqwest::Error) -> String {
    if error.is_timeout() {
        "模型请求超时（最长等待 20 分钟），请检查网络或降低单次处理量。".to_string()
    } else if error.is_connect() {
        format!("无法连接模型服务：{}", error)
    } else {
        error.to_string()
    }
}

pub(crate) async fn response_json_or_error(
    response: reqwest::Response,
) -> Result<(Value, String), ModelResponseError> {
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| ModelResponseError::other(format_request_error(error)))?;
    if !status.is_success() {
        return Err(ModelResponseError::provider(
            status.as_u16(),
            body,
            None,
        ));
    }
    let value: Value = serde_json::from_str(&body).map_err(|error| {
        ModelResponseError::other(format!(
            "模型响应不是合法 JSON: {}；原始响应：{}",
            error, body
        ))
    })?;
    Ok((value, body))
}

pub(crate) fn parse_gemini_parts(value: &Value) -> Result<(String, Option<String>), String> {
    let candidates = value
        .get("candidates")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Gemini 响应缺少 candidates 数组".to_string())?;
    let candidate = candidates
        .first()
        .ok_or_else(|| "Gemini 响应 candidates 为空".to_string())?;
    let parts = candidate
        .get("content")
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())
        .ok_or_else(|| "Gemini 响应缺少 content.parts".to_string())?;

    let mut text_parts = Vec::new();
    let mut reasoning_parts = Vec::new();

    for part in parts {
        if let Some(thought) = part.get("thought").and_then(|v| v.as_bool()) {
            if thought {
                if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                    reasoning_parts.push(text.to_string());
                }
                continue;
            }
        }
        if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
            text_parts.push(text.to_string());
        }
    }

    let text = text_parts.join("");
    let reasoning = if reasoning_parts.is_empty() {
        None
    } else {
        Some(reasoning_parts.join(""))
    };

    Ok((text, reasoning))
}

pub(crate) fn model_output_truncation_error(raw_response: &str) -> Option<String> {
    if raw_response.contains("length") || raw_response.contains("max_tokens") {
        Some("输出被截断".to_string())
    } else {
        None
    }
}
