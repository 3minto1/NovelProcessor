export type Novel = {
  id: string;
  title: string;
  source_path: string;
  encoding: string;
  status: string;
  created_at: string;
};

export type Chapter = {
  id: string;
  novel_id: string;
  index: number;
  title: string;
  original_text: string;
  is_valid: boolean;
  validation_reason?: string | null;
  corrected_text?: string | null;
  validation_status: string;
  review_status: string;
};

export type ChapterBatch = {
  id: string;
  novel_id: string;
  batch_index: number;
  label: string;
  start_chapter: number;
  end_chapter: number;
  status: string;
  created_at: string;
};

export type NovelDetail = {
  novel: Novel;
  chapters: Chapter[];
  batches: ChapterBatch[];
};

export type ModelProfile = {
  id: string;
  name: string;
  provider: string;
  base_url: string;
  model: string;
  temperature: number;
  top_p: number;
  thinking_mode: "auto" | "off" | "on";
  has_api_key: boolean;
  api_key_storage: "system" | "database_fallback" | "none";
  updated_at: string;
};

export type ProfileDraft = {
  id?: string;
  name: string;
  provider: string;
  base_url: string;
  model: string;
  temperature: number;
  top_p: number;
  thinking_mode: "auto" | "off" | "on";
  api_key: string;
};

export type ModelProfileInput = Omit<ProfileDraft, "api_key"> & { api_key?: string };

export type Job = {
  id: string;
  novel_id: string;
  job_type: string;
  status: string;
  current_chapter: number;
  total_chapters: number;
  message: string;
  phase?: "validate" | "review" | "export";
  batch_index?: number;
  batch_total?: number;
  batch_label?: string;
};

export type AiLog = {
  id: string;
  novel_id?: string | null;
  profile_id: string;
  action: string;
  chapter_title?: string | null;
  status: string;
  content: string;
  reasoning?: string | null;
  raw_response?: string | null;
  created_at: string;
};

export type AppSettings = {
  export_dir?: string | null;
  selected_profile_id?: string | null;
  chapter_batch_size?: 30 | 50 | 100;
};

export type JobEstimate = {
  novel_chapters: number;
  novel_chars: number;
  novel_batches: number;
  selected_batch_chapters: number;
  selected_batch_chars: number;
  parallelism: number;
  current_batch_requests: number;
  full_run_requests: number;
  average_call_seconds?: number | null;
  estimated_current_batch_seconds?: number | null;
  estimated_full_run_seconds?: number | null;
  recent_success_calls: number;
  recent_failed_calls: number;
  average_input_chars?: number | null;
  average_output_chars?: number | null;
};

export type DiagnosisStatus = "ok" | "warning" | "failed";

export type ModelDiagnosis = {
  status: DiagnosisStatus;
  recommended_thinking_mode?: "auto" | "off" | "on" | null;
  checks: Array<{
    name: string;
    status: DiagnosisStatus;
    message: string;
  }>;
};

export type ExportResult = { path: string };
