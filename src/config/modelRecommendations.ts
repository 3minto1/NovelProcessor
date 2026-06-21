import type { ProfileDraft } from "../types";

type ModelSuggestion = {
  label: string;
  model: string;
};

type ModelSuggestionGroup = {
  baseTerms: string[];
  modelTerms: string[];
  models: ModelSuggestion[];
};

export type ThinkingModeSupport = {
  disabledModes: Array<"off" | "on">;
  guidance: string;
};

export const emptyProfile: ProfileDraft = {
  name: "OpenAI 兼容接口",
  provider: "openai-compatible",
  base_url: "https://api.openai.com/v1",
  model: "请填写模型名",
  temperature: 0.7,
  top_p: 1,
  thinking_mode: "auto",
  api_key: ""
};

const groups: ModelSuggestionGroup[] = [
  {
    baseTerms: ["deepseek"],
    modelTerms: ["deepseek"],
    models: [
      { label: "DeepSeek V4 Pro", model: "deepseek-v4-pro" },
      { label: "DeepSeek V4 Flash", model: "deepseek-v4-flash" }
    ]
  },
  {
    baseTerms: ["volcengine", "volces", "ark.cn-"],
    modelTerms: ["doubao-", "seed-"],
    models: [
      { label: "Doubao Seed 2.0 Pro", model: "doubao-seed-2-0-pro-260215" },
      { label: "Doubao Seed 2.0 Lite", model: "doubao-seed-2-0-lite-260428" },
      { label: "Doubao Seed 2.0 Mini", model: "doubao-seed-2-0-mini-260428" },
      { label: "Doubao Seed 2.0 Code", model: "doubao-seed-2-0-code-preview-260215" },
      { label: "Doubao 1.5 Pro 32K", model: "doubao-1-5-pro-32k-250115" },
      { label: "Doubao 1.5 Pro 256K", model: "doubao-1-5-pro-256k-250115" },
      { label: "Doubao 1.5 Lite 32K", model: "doubao-1-5-lite-32k-250115" },
      { label: "Doubao 1.5 Thinking Pro", model: "doubao-1-5-thinking-pro-250415" },
      { label: "Doubao 1.5 Vision Pro", model: "doubao-1-5-vision-pro-250328" }
    ]
  },
  {
    baseTerms: ["api.openai.com", "openai.azure.com"],
    modelTerms: ["gpt-", "o3", "o4"],
    models: [
      { label: "GPT-5.2", model: "gpt-5.2" },
      { label: "GPT-5.2 Pro", model: "gpt-5.2-pro" },
      { label: "GPT-5.1", model: "gpt-5.1" },
      { label: "GPT-5", model: "gpt-5" },
      { label: "GPT-5 Mini", model: "gpt-5-mini" },
      { label: "GPT-5 Nano", model: "gpt-5-nano" },
      { label: "o3 Pro", model: "o3-pro" },
      { label: "o3", model: "o3" },
      { label: "GPT-4.1", model: "gpt-4.1" },
      { label: "GPT-4.1 Mini", model: "gpt-4.1-mini" },
      { label: "GPT-4o Mini", model: "gpt-4o-mini" }
    ]
  },
  {
    baseTerms: ["bigmodel", "zhipu", "z.ai"],
    modelTerms: ["glm-"],
    models: [
      { label: "GLM-5.2", model: "glm-5.2" },
      { label: "GLM-5.1", model: "glm-5.1" },
      { label: "GLM-5", model: "glm-5" },
      { label: "GLM-5 Turbo", model: "glm-5-turbo" },
      { label: "GLM-4.7", model: "glm-4.7" },
      { label: "GLM-4.6", model: "glm-4.6" },
      { label: "GLM-4.5", model: "glm-4.5" },
      { label: "GLM-4.5 Air", model: "glm-4.5-air" },
      { label: "GLM-4 Plus", model: "glm-4-plus" },
      { label: "GLM-4 Flash", model: "glm-4-flash" }
    ]
  },
  {
    baseTerms: ["moonshot", "kimi"],
    modelTerms: ["moonshot", "kimi"],
    models: [
      { label: "Kimi K2.6", model: "kimi-k2.6" },
      { label: "Kimi K2.5", model: "kimi-k2.5" },
      { label: "Moonshot V1 128K", model: "moonshot-v1-128k" },
      { label: "Moonshot V1 32K", model: "moonshot-v1-32k" },
      { label: "Moonshot V1 8K", model: "moonshot-v1-8k" }
    ]
  },
  {
    baseTerms: ["minimax"],
    modelTerms: ["minimax", "m2-her"],
    models: [
      { label: "MiniMax M3", model: "MiniMax-M3" },
      { label: "MiniMax M2.7", model: "MiniMax-M2.7" },
      { label: "MiniMax M2.5", model: "MiniMax-M2.5" },
      { label: "MiniMax M2", model: "MiniMax-M2" },
      { label: "M2-her", model: "M2-her" }
    ]
  },
  {
    baseTerms: ["xiaomimimo", "mimo.xiaomi", "mimo.mi.com", "mimo"],
    modelTerms: ["mimo-"],
    models: [
      { label: "MiMo V2.5 Pro", model: "mimo-v2.5-pro" },
      { label: "MiMo V2.5", model: "mimo-v2.5" },
      { label: "MiMo V2 Flash", model: "mimo-v2-flash" }
    ]
  },
  {
    baseTerms: ["siliconflow"],
    modelTerms: ["qwen/", "thudm/", "deepseek-ai/", "moonshotai/", "minimaxai/", "zai-org/", "bytedance-seed/", "internlm/", "mistralai/", "openai/"],
    models: [
      { label: "DeepSeek V3.2", model: "deepseek-ai/DeepSeek-V3.2" },
      { label: "DeepSeek R1", model: "deepseek-ai/DeepSeek-R1" },
      { label: "Qwen3.5 122B A10B", model: "Qwen/Qwen3.5-122B-A10B" },
      { label: "Kimi K2.6", model: "moonshotai/Kimi-K2.6" },
      { label: "GLM-5.1", model: "zai-org/GLM-5.1" }
    ]
  },
  {
    baseTerms: ["anthropic", "claude"],
    modelTerms: ["claude-"],
    models: [
      { label: "Claude Opus 4.8", model: "claude-opus-4-8" },
      { label: "Claude Sonnet 4.6", model: "claude-sonnet-4-6" },
      { label: "Claude Haiku 4.5", model: "claude-haiku-4-5-20251001" }
    ]
  }
];

export function getModelSuggestions(profile: ProfileDraft): ModelSuggestion[] {
  const baseHint = profile.base_url.toLowerCase();
  const modelHint = profile.model.toLowerCase();
  const baseMatched = groups.find((group) =>
    group.baseTerms.some((term) => baseHint.includes(term))
  );
  if (baseMatched) return baseMatched.models;
  return groups.find((group) =>
    group.modelTerms.some((term) => modelHint.includes(term))
  )?.models ?? [];
}

function includesAny(value: string, terms: string[]) {
  return terms.some((term) => value.includes(term));
}

function isSiliconFlowToggleModel(model: string) {
  return [
    "deepseek-ai/deepseek-v3.2",
    "deepseek-ai/deepseek-v3.1-terminus",
    "qwen/qwen3.5-122b-a10b",
    "qwen/qwen3.5-35b-a3b",
    "qwen/qwen3.5-27b"
  ].includes(model);
}

export function getThinkingModeSupport(profile: ProfileDraft): ThinkingModeSupport {
  const base = profile.base_url.trim().toLowerCase();
  const model = profile.model.trim().toLowerCase();
  const provider = profile.provider.trim().toLowerCase();

  if (provider === "gemini") {
    if (model.includes("2.5-pro")) {
      return {
        disabledModes: [],
        guidance: "Gemini 2.5 Pro 始终会思考。"
      };
    }
    if (model.includes("2.5")) {
      return {
        disabledModes: [],
        guidance: "Gemini 2.5 使用 thinkingBudget。"
      };
    }
    if (model.includes("gemini-3")) {
      return {
        disabledModes: [],
        guidance: "Gemini 3 使用 thinkingLevel。"
      };
    }
    return {
      disabledModes: ["off", "on"],
      guidance: "当前 Gemini 型号未确认支持可控思考参数。"
    };
  }

  if (base.includes("siliconflow")) {
    if (isSiliconFlowToggleModel(model)) {
      return {
        disabledModes: [],
        guidance: "SiliconFlow 为该模型提供 enable_thinking 开关。"
      };
    }
    return {
      disabledModes: ["off", "on"],
      guidance: "SiliconFlow 只对部分模型提供 enable_thinking。"
    };
  }

  if (includesAny(base, ["api.deepseek.com"]) || model.startsWith("deepseek-v4")) {
    return {
      disabledModes: [],
      guidance: "DeepSeek V4 支持 Thinking / Non-Thinking 双模式。"
    };
  }

  if (includesAny(base, ["volcengine", "volces", "ark.cn-"])) {
    if (model.replace(/\./g, "-").includes("doubao-seed-2-0")) {
      return {
        disabledModes: [],
        guidance: "豆包 Seed 2.0 支持通过 thinking.type 开启或关闭深度思考。"
      };
    }
    return {
      disabledModes: ["off", "on"],
      guidance: "当前豆包型号未提供可控思考开关。"
    };
  }

  if (includesAny(base, ["bigmodel", "zhipu", "z.ai"]) || model.startsWith("glm-")) {
    if (/^glm-(?:[5-9]|4\.(?:[5-9]|[1-9]\d))/.test(model)) {
      return {
        disabledModes: [],
        guidance: "GLM 4.5 及以上支持 thinking.type 开关。"
      };
    }
    return {
      disabledModes: ["off", "on"],
      guidance: "该 GLM 型号早于 4.5，未确认支持 thinking.type。"
    };
  }

  if (includesAny(base, ["moonshot", "kimi"]) || model.startsWith("kimi-")) {
    if (model.startsWith("kimi-k2.5") || model.startsWith("kimi-k2.6")) {
      return {
        disabledModes: [],
        guidance: "Kimi K2.5 / K2.6 支持 thinking.type 开关。"
      };
    }
    return {
      disabledModes: ["off", "on"],
      guidance: "Moonshot V1 等当前型号不支持思考模式开关。"
    };
  }

  if (base.includes("minimax") || model.includes("minimax") || model.startsWith("m2-")) {
    if (model.includes("minimax-m3")) {
      return {
        disabledModes: [],
        guidance: "MiniMax M3 支持关闭或 Adaptive Thinking。"
      };
    }
    return {
      disabledModes: ["off", "on"],
      guidance: "MiniMax M2.x 的 thinking 无法关闭。"
    };
  }

  if (includesAny(base, ["xiaomimimo", "mimo.mi.com", "mimo"]) || model.startsWith("mimo-")) {
    return {
      disabledModes: [],
      guidance: "小米 MiMo 的推荐型号支持 thinking.type 开关。"
    };
  }

  if (base.includes("api.openai.com") || base.includes("openai.azure.com")) {
    if (/^(?:gpt-5|o[134])/.test(model)) {
      return {
        disabledModes: [],
        guidance: "OpenAI 推理模型支持 reasoning_effort。"
      };
    }
    return {
      disabledModes: ["off", "on"],
      guidance: "GPT-4.1、GPT-4o 等非推理型号不支持 reasoning_effort。"
    };
  }

  if (base.includes("anthropic") || model.startsWith("claude-")) {
    return {
      disabledModes: ["off", "on"],
      guidance: "Claude 原生 API 使用 adaptive / extended thinking。"
    };
  }

  return {
    disabledModes: ["off", "on"],
    guidance: "当前兼容接口未确认支持哪种思考参数。"
  };
}

export function normalizeThinkingMode(profile: ProfileDraft): ProfileDraft {
  const support = getThinkingModeSupport(profile);
  if (
    profile.thinking_mode !== "auto"
    && support.disabledModes.includes(profile.thinking_mode)
  ) {
    return { ...profile, thinking_mode: "auto" };
  }
  return profile;
}
