import { invoke } from "@tauri-apps/api/core";
import type {
  ExportResult,
  Job,
  ModelProfile,
  Novel,
  NovelDetail,
} from "./types";

type CommandMap = {
  list_novels: { args?: undefined; result: Novel[] };
  get_novel_detail: { args: { novelId: string }; result: NovelDetail };
  import_txt: { args: { filePath: string }; result: Novel };
  delete_novel: { args: { novelId: string }; result: void };
  list_model_profiles: { args?: undefined; result: ModelProfile[] };
  save_model_profile: {
    args: {
      name: string;
      provider: string;
      base_url: string;
      model: string;
      temperature: number;
      top_p: number;
      thinking_mode: string;
      api_key?: string;
    };
    result: ModelProfile;
  };
  delete_model_profile: { args: { profileId: string }; result: void };
  start_validation: {
    args: { novelId: string; profileId: string };
    result: Job;
  };
  start_review: {
    args: { novelId: string; profileId: string };
    result: Job;
  };
  export_novel: {
    args: { novelId: string; outputDir: string };
    result: ExportResult;
  };
  get_job: {
    args: { jobId: string };
    result: Job;
  };
};

export type TauriCommand = keyof CommandMap;

export function invokeCommand<C extends TauriCommand>(
  command: C,
  ...args: CommandMap[C] extends { args: infer A } ? [args: A] : [args?: undefined]
): Promise<CommandMap[C]["result"]> {
  return invoke<CommandMap[C]["result"]>(command, args[0] as Record<string, unknown> | undefined);
}
