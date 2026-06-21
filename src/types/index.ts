export type Novel = {
  id: string;
  title: string;
  source_path: string;
  encoding: string;
  status: string;
  detected_chapters: boolean;
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
  api_key_storage: string;
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

export type Job = {
  id: string;
  novel_id: string;
  job_type: string;
  status: string;
  current_chapter: number;
  total_chapters: number;
  message: string;
  created_at: string;
  updated_at: string;
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
  chapter_batch_size?: number;
  selected_profile_id?: string | null;
};

export type NovelDetail = {
  novel: Novel;
  chapters: Chapter[];
  batches: ChapterBatch[];
};

export type ExportResult = { path: string };
