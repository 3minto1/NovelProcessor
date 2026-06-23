use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Novel {
    pub id: String,
    pub title: String,
    pub source_path: String,
    pub encoding: String,
    pub status: String,
    pub detected_chapters: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub novel_id: String,
    pub index: i64,
    pub title: String,
    pub original_text: String,
    pub is_valid: bool,
    pub validation_reason: Option<String>,
    pub corrected_text: Option<String>,
    pub validation_status: String,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterBatch {
    pub id: String,
    pub novel_id: String,
    pub batch_index: i64,
    pub label: String,
    pub start_chapter: i64,
    pub end_chapter: i64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub temperature: f64,
    pub top_p: f64,
    pub thinking_mode: String,
    pub has_api_key: bool,
    pub api_key_storage: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub novel_id: String,
    pub job_type: String,
    pub status: String,
    pub current_chapter: i64,
    pub total_chapters: i64,
    pub message: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiLog {
    pub id: String,
    pub novel_id: Option<String>,
    pub profile_id: String,
    pub action: String,
    pub chapter_title: Option<String>,
    pub status: String,
    pub content: String,
    pub reasoning: Option<String>,
    pub raw_response: Option<String>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub export_dir: Option<String>,
    pub chapter_batch_size: i64,
    pub review_parallelism: i64,
    pub selected_profile_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SplitResult {
    pub chapters: Vec<Chapter>,
    pub detected_chapters: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelDetail {
    pub novel: Novel,
    pub chapters: Vec<Chapter>,
    pub batches: Vec<ChapterBatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOutput {
    pub text: String,
    pub reasoning: Option<String>,
    pub raw_response: String,
    pub input_chars: usize,
    pub output_chars: usize,
    pub elapsed_ms: u128,
    pub retried_without_thinking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisCheck {
    pub name: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDiagnosis {
    pub status: String,
    pub recommended_thinking_mode: Option<String>,
    pub checks: Vec<DiagnosisCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfileInput {
    pub id: Option<String>,
    pub name: String,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub temperature: f64,
    pub top_p: f64,
    pub thinking_mode: String,
    pub api_key: Option<String>,
}
