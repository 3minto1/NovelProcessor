import { describe, it, expect } from "vitest";
import {
  emptyProfile,
  getModelSuggestions,
  getThinkingModeSupport,
  normalizeThinkingMode,
} from "./modelRecommendations";
import type { ProfileDraft } from "../types";

function makeProfile(overrides: Partial<ProfileDraft> = {}): ProfileDraft {
  return { ...emptyProfile, ...overrides };
}

describe("getModelSuggestions", () => {
  it("returns DeepSeek models for deepseek base_url", () => {
    const suggestions = getModelSuggestions(
      makeProfile({ base_url: "https://api.deepseek.com/v1", model: "" })
    );
    expect(suggestions.length).toBeGreaterThan(0);
    expect(suggestions.some((s) => s.model.includes("deepseek"))).toBe(true);
  });

  it("returns Doubao models for volcengine base_url", () => {
    const suggestions = getModelSuggestions(
      makeProfile({ base_url: "https://ark.cn-beijing.volces.com/api/v3", model: "" })
    );
    expect(suggestions.length).toBeGreaterThan(0);
    expect(suggestions.some((s) => s.model.includes("doubao"))).toBe(true);
  });

  it("returns OpenAI models for api.openai.com base_url", () => {
    const suggestions = getModelSuggestions(
      makeProfile({ base_url: "https://api.openai.com/v1", model: "" })
    );
    expect(suggestions.length).toBeGreaterThan(0);
    expect(suggestions.some((s) => s.model.includes("gpt"))).toBe(true);
  });

  it("returns empty array for gemini base_url (no model list configured)", () => {
    const suggestions = getModelSuggestions(
      makeProfile({ base_url: "https://generativelanguage.googleapis.com", model: "" })
    );
    expect(suggestions).toEqual([]);
  });

  it("falls back to model term matching when base_url has no match", () => {
    const suggestions = getModelSuggestions(
      makeProfile({ base_url: "https://custom-api.example.com/v1", model: "deepseek-v4-pro" })
    );
    expect(suggestions.length).toBeGreaterThan(0);
    expect(suggestions.some((s) => s.model.includes("deepseek"))).toBe(true);
  });

  it("returns empty array when nothing matches", () => {
    const suggestions = getModelSuggestions(
      makeProfile({ base_url: "https://unknown.example.com", model: "unknown-model" })
    );
    expect(suggestions).toEqual([]);
  });

  it("returns MiMo models for xiaomimimo base_url", () => {
    const suggestions = getModelSuggestions(
      makeProfile({ base_url: "https://mimo.xiaomi.com/v1", model: "" })
    );
    expect(suggestions.length).toBeGreaterThan(0);
    expect(suggestions.some((s) => s.model.includes("mimo"))).toBe(true);
  });
});

describe("getThinkingModeSupport", () => {
  it("returns disabled off/on for generic provider", () => {
    const support = getThinkingModeSupport(
      makeProfile({ base_url: "https://custom-api.example.com", model: "custom-model" })
    );
    expect(support.disabledModes).toContain("off");
    expect(support.disabledModes).toContain("on");
    expect(support.guidance).toBeTruthy();
  });

  it("allows all modes for DeepSeek V4", () => {
    const support = getThinkingModeSupport(
      makeProfile({ base_url: "https://api.deepseek.com/v1", model: "deepseek-v4-pro" })
    );
    expect(support.disabledModes).not.toContain("off");
    expect(support.disabledModes).not.toContain("on");
  });

  it("allows all modes for Gemini 2.5 Pro", () => {
    const support = getThinkingModeSupport(
      makeProfile({ provider: "gemini", model: "gemini-2.5-pro" })
    );
    expect(support.disabledModes).not.toContain("off");
    expect(support.disabledModes).not.toContain("on");
  });

  it("allows all modes for Doubao Seed 2.0", () => {
    const support = getThinkingModeSupport(
      makeProfile({ base_url: "https://ark.cn-beijing.volces.com/api/v3", model: "doubao-seed-2-0-pro" })
    );
    expect(support.disabledModes).not.toContain("off");
    expect(support.disabledModes).not.toContain("on");
  });

  it("allows all modes for Kimi K2.5", () => {
    const support = getThinkingModeSupport(
      makeProfile({ base_url: "https://api.moonshot.cn/v1", model: "kimi-k2.5" })
    );
    expect(support.disabledModes).not.toContain("off");
    expect(support.disabledModes).not.toContain("on");
  });

  it("disables off/on for Claude", () => {
    const support = getThinkingModeSupport(
      makeProfile({ base_url: "https://api.anthropic.com", model: "claude-sonnet-4-6" })
    );
    expect(support.disabledModes).toContain("off");
    expect(support.disabledModes).toContain("on");
  });

  it("disables off/on for GPT-4o (non-reasoning model)", () => {
    const support = getThinkingModeSupport(
      makeProfile({ base_url: "https://api.openai.com/v1", model: "gpt-4o" })
    );
    expect(support.disabledModes).toContain("off");
    expect(support.disabledModes).toContain("on");
  });

  it("allows all modes for o3 (reasoning model)", () => {
    const support = getThinkingModeSupport(
      makeProfile({ base_url: "https://api.openai.com/v1", model: "o3" })
    );
    expect(support.disabledModes).not.toContain("off");
    expect(support.disabledModes).not.toContain("on");
  });

  it("allows all modes for MiMo models", () => {
    const support = getThinkingModeSupport(
      makeProfile({ base_url: "https://mimo.xiaomi.com/v1", model: "mimo-v2.5-pro" })
    );
    expect(support.disabledModes).not.toContain("off");
    expect(support.disabledModes).not.toContain("on");
  });
});

describe("normalizeThinkingMode", () => {
  it("returns profile unchanged when mode is auto", () => {
    const profile = makeProfile({ thinking_mode: "auto" });
    const result = normalizeThinkingMode(profile);
    expect(result.thinking_mode).toBe("auto");
  });

  it("resets unsupported mode to auto", () => {
    const profile = makeProfile({
      base_url: "https://api.anthropic.com",
      model: "claude-sonnet-4-6",
      thinking_mode: "off",
    });
    const result = normalizeThinkingMode(profile);
    expect(result.thinking_mode).toBe("auto");
  });

  it("keeps supported mode unchanged", () => {
    const profile = makeProfile({
      base_url: "https://api.deepseek.com/v1",
      model: "deepseek-v4-pro",
      thinking_mode: "off",
    });
    const result = normalizeThinkingMode(profile);
    expect(result.thinking_mode).toBe("off");
  });
});
