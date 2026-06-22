import { invoke } from "@tauri-apps/api/core";
import type {
  AiLog,
  AppSettings,
  ExportResult,
  Job,
  ModelDiagnosis,
  ModelProfile,
  Novel,
  NovelDetail,
} from "./types";

type CommandMap = {
  list_novels: { args?: undefined; result: Novel[] };
  get_novel_detail: { args: { novelId: string }; result: NovelDetail };
  import_txt: { args: { filePath: string }; result: Novel };
  delete_novel: { args: { novelId: string }; result: void };
  update_chapter_text: {
    args: { chapterId: string; title: string; originalText: string };
    result: void;
  };
  delete_chapter: { args: { chapterId: string }; result: void };
  toggle_chapter_validity: {
    args: { chapterId: string; isValid: boolean };
    result: void;
  };
  list_model_profiles: { args?: undefined; result: ModelProfile[] };
  save_model_profile: {
    args: {
      input: {
        id?: string;
        name: string;
        provider: string;
        base_url: string;
        model: string;
        temperature: number;
        top_p: number;
        thinking_mode: string;
        api_key?: string;
      };
    };
    result: ModelProfile;
  };
  delete_model_profile: { args: { profileId: string }; result: void };
  diagnose_model_profile: { args: { profileId: string }; result: ModelDiagnosis };
  list_ai_logs: { args: { novelId: string | null }; result: AiLog[] };
  clear_ai_logs: { args: { novelId: string | null }; result: void };
  get_app_settings: { args?: undefined; result: AppSettings };
  save_app_settings: { args: { settings: AppSettings }; result: AppSettings };
  save_selected_profile_id: { args: { profileId: string | null }; result: AppSettings };
  start_validation: { args: { novelId: string; profileId: string }; result: Job };
  cancel_validation: { args?: undefined; result: void };
  is_validation_active: { args?: undefined; result: boolean };
  start_review: { args: { novelId: string; profileId: string }; result: Job };
  export_novel: { args: { novelId: string; outputDir: string }; result: ExportResult };
  get_job: { args: { jobId: string }; result: Job };
  record_frontend_error: {
    args: { message: string; stack: string | null; componentStack: string | null };
    result: void;
  };
};

export type TauriCommand = keyof CommandMap;

export function invokeCommand<C extends TauriCommand>(
  command: C,
  ...args: CommandMap[C] extends { args: infer A } ? [args: A] : [args?: undefined]
): Promise<CommandMap[C]["result"]> {
  return invoke<CommandMap[C]["result"]>(command, args[0] as Record<string, unknown> | undefined);
}
