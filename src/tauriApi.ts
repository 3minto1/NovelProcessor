import { invoke } from "@tauri-apps/api/core";
import type {
  AiLog,
  AppSettings,
  ExportResult,
  Job,
  JobEstimate,
  ModelDiagnosis,
  ModelProfile,
  ModelProfileInput,
  Novel,
  NovelDetail,
} from "./types";

type CommandMap = {
  list_novels: { args?: undefined; result: Novel[] };
  get_novel_detail: { args: { novelId: string }; result: NovelDetail };
  import_txt: { args: { filePath: string }; result: Novel };
  delete_novel: { args: { novelId: string }; result: void };
  list_model_profiles: { args?: undefined; result: ModelProfile[] };
  save_model_profile: { args: { input: ModelProfileInput }; result: ModelProfile };
  delete_model_profile: { args: { profileId: string }; result: void };
  diagnose_model_profile: { args: { profileId: string }; result: ModelDiagnosis };
  list_ai_logs: { args: { novelId: string | null }; result: AiLog[] };
  clear_ai_logs: { args: { novelId: string | null }; result: void };
  get_app_settings: { args?: undefined; result: AppSettings };
  save_app_settings: { args: { settings: AppSettings }; result: AppSettings };
  save_selected_profile_id: { args: { profileId: string | null }; result: AppSettings };
  estimate_job_cost: {
    args: { novelId: string; batchId: string | null; profileId: string | null };
    result: JobEstimate;
  };
  start_validation: { args: { novelId: string; profileId: string }; result: Job };
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
