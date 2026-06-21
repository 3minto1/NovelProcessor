import { useCallback } from "react";
import { useAppStore } from "../store/appStore";
import { invokeCommand } from "../tauriApi";

const defaultProfile = {
  name: "OpenAI 兼容接口",
  provider: "openai-compatible",
  base_url: "https://api.openai.com/v1",
  model: "请填写模型名",
  temperature: 0.7,
  top_p: 1,
  thinking_mode: "auto" as const,
  api_key: "",
};

export function useModelProfiles() {
  const {
    profiles,
    setProfiles,
    selectedProfileId,
    setSelectedProfileId,
  } = useAppStore();

  const selectedProfile = profiles.find((p) => p.id === selectedProfileId);

  const refreshProfiles = useCallback(async () => {
    const rows = await invokeCommand("list_model_profiles");
    setProfiles(rows);
    if (!selectedProfileId || !rows.some((p) => p.id === selectedProfileId)) {
      setSelectedProfileId(rows[0]?.id ?? "");
    }
  }, [selectedProfileId, setProfiles, setSelectedProfileId]);

  const saveProfile = useCallback(
    async (draft: typeof defaultProfile & { id?: string }) => {
      const saved = await invokeCommand("save_model_profile", {
        name: draft.name,
        provider: draft.provider,
        base_url: draft.base_url,
        model: draft.model,
        temperature: draft.temperature,
        top_p: draft.top_p,
        thinking_mode: draft.thinking_mode,
        api_key: draft.api_key,
      });
      await refreshProfiles();
      return saved;
    },
    [refreshProfiles]
  );

  const deleteProfile = useCallback(
    async (profileId: string) => {
      await invokeCommand("delete_model_profile", { profileId });
      await refreshProfiles();
    },
    [refreshProfiles]
  );

  return {
    profiles,
    selectedProfileId,
    setSelectedProfileId,
    selectedProfile,
    refreshProfiles,
    saveProfile,
    deleteProfile,
    defaultProfile,
  };
}
