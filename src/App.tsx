import { getCurrentWebview, type DragDropEvent } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import {
  BookOpen,
  CheckCircle2,
  FileText,
  Loader2,
  Play,
  Settings,
  Trash2,
  XCircle,
} from "lucide-react";
import { useEffect, useState } from "react";
import { ErrorBoundary } from "./components/common/ErrorBoundary";
import { useModelProfiles } from "./hooks/useModelProfiles";
import { useNovels } from "./hooks/useNovels";
import { useNotice } from "./hooks/useNotice";
import { useAppStore } from "./store/appStore";
import { invokeCommand as invoke } from "./tauriApi";
import type { Job, Novel } from "./types";

type WorkflowStep = "import" | "validate" | "review" | "export";

export default function App() {
  const { novels, setNovels, detail, setDetail, busy, setBusy, job, setJob, setProfiles } =
    useAppStore();
  const { notice, setNotice, showNotice } = useNotice();
  const {
    profiles,
    selectedProfileId,
    setSelectedProfileId,
  } = useModelProfiles();
  const { loadNovel } = useNovels();

  const [activeStep, setActiveStep] = useState<WorkflowStep>("import");
  const [dragActive, setDragActive] = useState(false);
  const [showModelDialog, setShowModelDialog] = useState(false);
  const [modelDraft, setModelDraft] = useState({
    name: "",
    provider: "openai-compatible",
    base_url: "https://api.openai.com/v1",
    model: "",
    temperature: 0.7,
    top_p: 1.0,
    thinking_mode: "auto" as const,
    api_key: "",
  });

  useEffect(() => {
    void refreshAll();
  }, []);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    function handleDragDrop(event: { payload: DragDropEvent }) {
      const payload = event.payload;
      if (payload.type === "enter") {
        setDragActive(payload.paths.some((p) => p.endsWith(".txt")));
        return;
      }
      if (payload.type === "leave") {
        setDragActive(false);
        return;
      }
      if (payload.type !== "drop") return;
      setDragActive(false);
      const txtPath = payload.paths.find((p) => p.endsWith(".txt"));
      if (txtPath) {
        void importTxtFile(txtPath);
      }
    }

    void getCurrentWebview().onDragDropEvent(handleDragDrop).then((handler) => {
      if (cancelled) {
        handler();
      } else {
        unlisten = handler;
      }
    });

    return () => {
      cancelled = true;
      setDragActive(false);
      if (unlisten) unlisten();
    };
  }, []);

  // Poll for job progress
  useEffect(() => {
    if (!job || job.status === "completed" || job.status === "failed") {
      return;
    }
    
    const interval = setInterval(async () => {
      try {
        const updatedJob: Job = await invoke("get_job", { jobId: job.id });
        setJob(updatedJob);
        
        if (updatedJob.status === "completed" || updatedJob.status === "failed") {
          clearInterval(interval);
          // Refresh novel data
          if (detail) {
            await loadNovel(detail.novel.id);
          }
        }
      } catch {
        // Ignore polling errors
      }
    }, 2000);
    
    return () => clearInterval(interval);
  }, [job?.id, job?.status]);

  async function refreshAll() {
    const [novelRows, profileRows] = await Promise.all([
      invoke("list_novels"),
      invoke("list_model_profiles"),
    ]);
    setNovels(novelRows);
    setProfiles(profileRows);
    if (novelRows[0]) {
      await loadNovel(novelRows[0].id);
    }
  }

  async function importTxtFile(filePath: string) {
    if (!filePath.endsWith(".txt")) {
      showNotice("当前仅支持导入 TXT 小说文件。");
      return;
    }
    setBusy("import");
    setNotice("");
    try {
      const novel = await invoke("import_txt", { filePath });
      await refreshAll();
      await loadNovel(novel.id);
      showNotice(`已导入《${novel.title}》。`);
    } catch (error) {
      showNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function importTxt() {
    setBusy("import");
    setNotice("");
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "TXT 小说", extensions: ["txt"] }],
      });
      if (typeof selected !== "string") return;
      await importTxtFile(selected);
    } catch (error) {
      showNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function deleteNovel(novel: Novel) {
    if (!window.confirm(`删除《${novel.title}》？`)) return;
    setBusy("delete-novel");
    setNotice("");
    try {
      await invoke("delete_novel", { novelId: novel.id });
      const remaining = await invoke("list_novels");
      setNovels(remaining);
      if (detail?.novel.id === novel.id) {
        if (remaining[0]) {
          await loadNovel(remaining[0].id);
        } else {
          setDetail(null);
        }
      }
      showNotice(`已删除《${novel.title}》。`);
    } catch (error) {
      showNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function runValidation() {
    if (!detail || !selectedProfileId) {
      showNotice("请先导入小说并选择模型配置。");
      return;
    }
    setBusy("validate");
    setNotice("");
    try {
      const result = await invoke("start_validation", {
        novelId: detail.novel.id,
        profileId: selectedProfileId,
      });
      setJob(result);
      await loadNovel(detail.novel.id);
      showNotice(result.message);
    } catch (error) {
      showNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function runReview() {
    if (!detail || !selectedProfileId) {
      showNotice("请先导入小说并选择模型配置。");
      return;
    }
    setBusy("review");
    setNotice("");
    try {
      const result = await invoke("start_review", {
        novelId: detail.novel.id,
        profileId: selectedProfileId,
      });
      setJob(result);
      await loadNovel(detail.novel.id);
      showNotice(result.message);
    } catch (error) {
      showNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function exportNovel() {
    if (!detail) return;
    setBusy("export");
    setNotice("");
    try {
      const selected = await open({ directory: true, multiple: false });
      if (typeof selected !== "string") return;
      const result = await invoke("export_novel", {
        novelId: detail.novel.id,
        outputDir: selected,
      });
      showNotice(`已导出：${result.path}`);
    } catch (error) {
      showNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function saveModelProfile() {
    setBusy("save-model");
    setNotice("");
    try {
      await invoke("save_model_profile", {
        name: modelDraft.name,
        provider: modelDraft.provider,
        base_url: modelDraft.base_url,
        model: modelDraft.model,
        temperature: modelDraft.temperature,
        top_p: modelDraft.top_p,
        thinking_mode: modelDraft.thinking_mode,
        api_key: modelDraft.api_key || undefined,
      });
      await refreshAll();
      setShowModelDialog(false);
      setModelDraft({
        name: "",
        provider: "openai-compatible",
        base_url: "https://api.openai.com/v1",
        model: "",
        temperature: 0.7,
        top_p: 1.0,
        thinking_mode: "auto",
        api_key: "",
      });
      showNotice("模型配置已保存。");
    } catch (error) {
      showNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  const validChapters = detail?.chapters.filter((c) => c.is_valid) ?? [];
  const invalidChapters = detail?.chapters.filter((c) => !c.is_valid) ?? [];

  return (
    <ErrorBoundary>
      <div className="app">
        <header className="app-header">
          <h1>NovelProcessor</h1>
          <div className="header-actions">
            <select
              value={selectedProfileId}
              onChange={(e) => setSelectedProfileId(e.target.value)}
            >
              <option value="">选择模型</option>
              {profiles.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
            <button onClick={() => setShowModelDialog(true)}>
              <Settings size={14} />
              添加模型
            </button>
          </div>
        </header>

        <div className="app-content">
          <aside className="sidebar">
            <div className="sidebar-section">
              <h3>小说列表</h3>
              <button onClick={importTxt} disabled={!!busy}>
                <FileText size={14} />
                导入TXT
              </button>
              <ul className="novel-list">
                {novels.map((novel) => (
                  <li
                    key={novel.id}
                    className={detail?.novel.id === novel.id ? "active" : ""}
                  >
                    <span onClick={() => loadNovel(novel.id)}>
                      {novel.title}
                    </span>
                    <button
                      onClick={() => deleteNovel(novel)}
                      className="icon-button"
                    >
                      <Trash2 size={14} />
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          </aside>

          <main className="main-content">
            {detail ? (
              <>
                <div className="workflow-steps">
                  <button
                    className={activeStep === "import" ? "active" : ""}
                    onClick={() => setActiveStep("import")}
                  >
                    <FileText size={16} />
                    导入
                  </button>
                  <button
                    className={activeStep === "validate" ? "active" : ""}
                    onClick={() => setActiveStep("validate")}
                  >
                    <CheckCircle2 size={16} />
                    验证
                  </button>
                  <button
                    className={activeStep === "review" ? "active" : ""}
                    onClick={() => setActiveStep("review")}
                  >
                    <BookOpen size={16} />
                    审查
                  </button>
                  <button
                    className={activeStep === "export" ? "active" : ""}
                    onClick={() => setActiveStep("export")}
                  >
                    <Play size={16} />
                    导出
                  </button>
                </div>

                <div className="step-content">
                  {activeStep === "import" && (
                    <div className="import-step">
                      <h2>导入小说</h2>
                      <div className="stats-grid">
                        <div className="stat-card">
                          <div className="value">{detail.chapters.length}</div>
                          <div className="label">总章节数</div>
                        </div>
                        <div className="stat-card">
                          <div className="value">{validChapters.length}</div>
                          <div className="label">有效章节</div>
                        </div>
                        <div className="stat-card">
                          <div className="value">{invalidChapters.length}</div>
                          <div className="label">无效章节</div>
                        </div>
                        <div className="stat-card">
                          <div className="value">{detail.novel.encoding}</div>
                          <div className="label">编码格式</div>
                        </div>
                      </div>
                      <div className="chapter-list">
                        {detail.chapters.slice(0, 30).map((chapter) => (
                          <div key={chapter.id} className="chapter-item">
                            <span>{chapter.title}</span>
                            <span style={{ fontSize: '12px', color: '#999' }}>
                              {chapter.original_text.length} 字
                            </span>
                          </div>
                        ))}
                        {detail.chapters.length > 30 && (
                          <div className="chapter-item" style={{ color: '#999' }}>
                            ... 还有 {detail.chapters.length - 30} 章
                          </div>
                        )}
                      </div>
                    </div>
                  )}

                  {activeStep === "validate" && (
                    <div className="validate-step">
                      <h2>AI验证章节</h2>
                      <p>将发送章节给AI判断是否为有效小说内容，剔除广告、作者笔记、乱码等无效内容</p>
                      <button
                        onClick={runValidation}
                        disabled={!!busy || !selectedProfileId}
                        className="primary-button"
                      >
                        {busy === "validate" ? (
                          <Loader2 className="spin" size={16} />
                        ) : (
                          <CheckCircle2 size={16} />
                        )}
                        开始验证
                      </button>
                      {!selectedProfileId && (
                        <p style={{ color: '#f44336', marginTop: '8px' }}>
                          请先选择一个模型配置
                        </p>
                      )}
                      {job && job.job_type === "validate" && (
                        <div className="job-progress">
                          <p>{job.message}</p>
                          <div className="progress-bar">
                            <div
                              className="progress-fill"
                              style={{
                                width: `${job.total_chapters > 0 ? (job.current_chapter / job.total_chapters) * 100 : 0}%`,
                              }}
                            />
                          </div>
                          <p style={{ fontSize: '12px', color: '#999', marginTop: '8px' }}>
                            {job.current_chapter} / {job.total_chapters}
                          </p>
                        </div>
                      )}
                      {invalidChapters.length > 0 && (
                        <div className="validation-results">
                          <h3 style={{ marginTop: '20px', marginBottom: '12px' }}>
                            验证结果 ({invalidChapters.length} 个无效章节)
                          </h3>
                          {invalidChapters.slice(0, 10).map((chapter) => (
                            <div key={chapter.id} className="validation-item invalid">
                              <XCircle size={16} color="#f44336" className="icon" />
                              <div className="content">
                                <div className="title">{chapter.title}</div>
                                <div className="reason">
                                  {chapter.validation_reason || "AI判定为无效内容"}
                                </div>
                              </div>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  )}

                  {activeStep === "review" && (
                    <div className="review-step">
                      <h2>AI审查章节</h2>
                      <p>检查错别字、无关内容和语法问题，并自动修正</p>
                      <button
                        onClick={runReview}
                        disabled={!!busy || !selectedProfileId}
                        className="primary-button"
                      >
                        {busy === "review" ? (
                          <Loader2 className="spin" size={16} />
                        ) : (
                          <BookOpen size={16} />
                        )}
                        开始审查
                      </button>
                      {!selectedProfileId && (
                        <p style={{ color: '#f44336', marginTop: '8px' }}>
                          请先选择一个模型配置
                        </p>
                      )}
                      {job && job.job_type === "review" && (
                        <div className="job-progress">
                          <p>{job.message}</p>
                          <div className="progress-bar">
                            <div
                              className="progress-fill"
                              style={{
                                width: `${job.total_chapters > 0 ? (job.current_chapter / job.total_chapters) * 100 : 0}%`,
                              }}
                            />
                          </div>
                          <p style={{ fontSize: '12px', color: '#999', marginTop: '8px' }}>
                            {job.current_chapter} / {job.total_chapters}
                          </p>
                        </div>
                      )}
                      <div style={{ marginTop: '20px' }}>
                        <p>有效章节: {validChapters.length} 章</p>
                      </div>
                    </div>
                  )}

                  {activeStep === "export" && (
                    <div className="export-step">
                      <h2>导出小说</h2>
                      <p>将修正后的小说导出为TXT文件</p>
                      <div className="stats-grid">
                        <div className="stat-card">
                          <div className="value">{validChapters.length}</div>
                          <div className="label">导出章节数</div>
                        </div>
                        <div className="stat-card">
                          <div className="value">
                            {validChapters.reduce((sum, c) => sum + (c.corrected_text || c.original_text).length, 0).toLocaleString()}
                          </div>
                          <div className="label">总字数</div>
                        </div>
                      </div>
                      <button
                        onClick={exportNovel}
                        disabled={!!busy || validChapters.length === 0}
                        className="primary-button"
                        style={{ marginTop: '20px' }}
                      >
                        {busy === "export" ? (
                          <Loader2 className="spin" size={16} />
                        ) : (
                          <Play size={16} />
                        )}
                        导出
                      </button>
                      {validChapters.length === 0 && (
                        <p style={{ color: '#f44336', marginTop: '8px' }}>
                          没有可导出的有效章节
                        </p>
                      )}
                    </div>
                  )}
                </div>
              </>
            ) : (
              <div className="empty-state">
                <BookOpen size={48} />
                <h2>欢迎使用 NovelProcessor</h2>
                <p>请导入TXT小说文件开始处理</p>
                <button onClick={importTxt}>
                  <FileText size={16} />
                  导入小说
                </button>
              </div>
            )}
          </main>
        </div>

        {notice && <div className="notice">{notice}</div>}

        {dragActive && (
          <div className="drag-overlay">
            <p>拖放TXT文件到此处</p>
          </div>
        )}

        {showModelDialog && (
          <dialog open className="modal">
            <div className="modal-header">
              <h2>添加模型配置</h2>
              <button className="icon-button" onClick={() => setShowModelDialog(false)}>
                <XCircle size={18} />
              </button>
            </div>
            <div className="modal-body">
              <div style={{ marginBottom: '12px' }}>
                <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', fontWeight: 500 }}>
                  配置名称
                </label>
                <input
                  type="text"
                  value={modelDraft.name}
                  onChange={(e) => setModelDraft({ ...modelDraft, name: e.target.value })}
                  placeholder="例如: DeepSeek V3"
                  style={{ width: '100%', padding: '8px 12px', border: '1px solid #d0d0d0', borderRadius: '4px', fontSize: '13px' }}
                />
              </div>
              <div style={{ marginBottom: '12px' }}>
                <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', fontWeight: 500 }}>
                  API 地址
                </label>
                <input
                  type="text"
                  value={modelDraft.base_url}
                  onChange={(e) => setModelDraft({ ...modelDraft, base_url: e.target.value })}
                  placeholder="https://api.openai.com/v1"
                  style={{ width: '100%', padding: '8px 12px', border: '1px solid #d0d0d0', borderRadius: '4px', fontSize: '13px' }}
                />
              </div>
              <div style={{ marginBottom: '12px' }}>
                <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', fontWeight: 500 }}>
                  模型名称
                </label>
                <input
                  type="text"
                  value={modelDraft.model}
                  onChange={(e) => setModelDraft({ ...modelDraft, model: e.target.value })}
                  placeholder="gpt-4o-mini"
                  style={{ width: '100%', padding: '8px 12px', border: '1px solid #d0d0d0', borderRadius: '4px', fontSize: '13px' }}
                />
              </div>
              <div style={{ marginBottom: '12px' }}>
                <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', fontWeight: 500 }}>
                  API Key
                </label>
                <input
                  type="password"
                  value={modelDraft.api_key}
                  onChange={(e) => setModelDraft({ ...modelDraft, api_key: e.target.value })}
                  placeholder="sk-..."
                  style={{ width: '100%', padding: '8px 12px', border: '1px solid #d0d0d0', borderRadius: '4px', fontSize: '13px' }}
                />
              </div>
              <div style={{ display: 'flex', gap: '12px', marginTop: '20px' }}>
                <button onClick={() => setShowModelDialog(false)}>取消</button>
                <button
                  className="primary-button"
                  onClick={saveModelProfile}
                  disabled={!modelDraft.name || !modelDraft.model || !!busy}
                >
                  保存
                </button>
              </div>
            </div>
          </dialog>
        )}
      </div>
    </ErrorBoundary>
  );
}
