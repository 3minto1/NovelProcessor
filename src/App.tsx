import { getCurrentWebview, type DragDropEvent } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import {
  ArrowLeft,
  BookOpen,
  CheckCircle2,
  ClipboardList,
  Download,
  FilePlus2,
  GitCompareArrows,
  Loader2,
  MoreHorizontal,
  Square,
  Trash2,
  X
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { DeleteNovelDialog } from "./components/common/DeleteNovelDialog";
import { getStatusTone, StatusBadge } from "./components/common/StatusBadge";
import { LogsPage } from "./components/pages/LogsPage";
import { CompareView } from "./components/Compare/CompareView";
import { ModelProfiles } from "./components/Settings/ModelProfiles";
import { BatchPanel } from "./components/Workspace/BatchPanel";
import { ChapterList } from "./components/Workspace/ChapterList";
import { ModelConfig } from "./components/Workspace/ModelConfig";
import {
  emptyProfile as defaultProfile,
  getModelSuggestions as detectModelSuggestions
} from "./config/modelRecommendations";
import { useModelProfiles } from "./hooks/useModelProfiles";
import { useNovels } from "./hooks/useNovels";
import { useNotice } from "./hooks/useNotice";
import { useTaskState } from "./hooks/useTaskState";
import { invokeCommand as invoke } from "./tauriApi";
import type {
  AppSettings,
  Chapter,
  Job,
  ModelDiagnosis,
  Novel,
  NovelDetail
} from "./types";

type View = "workspace" | "settings" | "logs" | "compare";

const savedApiKeyMask = "********";

export default function App() {
  const {
    novels, setNovels, detail, setDetail, selectedChapterId, setSelectedChapterId,
    selectedBatchId, setSelectedBatchId
  } = useNovels();
  const {
    profiles, setProfiles, profileDraft, setProfileDraft,
    selectedProfileId, setSelectedProfileId, selectedProfile
  } = useModelProfiles(defaultProfile);
  const {
    busy, setBusy, job, setJob, processingTaskActive
  } = useTaskState();
  const { notice, setNotice, showNotice } = useNotice();

  const [activeView, setActiveView] = useState<View>("workspace");
  const [openNovelMenuId, setOpenNovelMenuId] = useState("");
  const [openModelMenu, setOpenModelMenu] = useState(false);
  const [openModelSuggestions, setOpenModelSuggestions] = useState(false);
  const [novelPendingDeletion, setNovelPendingDeletion] = useState<Novel | null>(null);
  const [settings, setSettings] = useState<AppSettings>({});
  const [modelDiagnosis, setModelDiagnosis] = useState<ModelDiagnosis | null>(null);
  const [dragActive, setDragActive] = useState(false);
  const [logs, setLogs] = useState<any[]>([]);
  const [taskPaused, setTaskPaused] = useState(false);
  const detailRef = useRef<NovelDetail | null>(null);
  const originalRef = useRef<HTMLDivElement>(null);
  const correctedRef = useRef<HTMLDivElement>(null);
  const busyRef = useRef("");
  const importInProgressRef = useRef(false);

  const detectedModelSuggestions = useMemo(
    () => detectModelSuggestions(profileDraft),
    [profileDraft.provider, profileDraft.base_url, profileDraft.model]
  );

  useEffect(() => {
    void refreshAll();
  }, []);

  useEffect(() => {
    busyRef.current = busy;
  }, [busy]);

  useEffect(() => {
    detailRef.current = detail;
  }, [detail]);

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

  // Poll for job progress and task pause state
  useEffect(() => {
    if (!job || job.status === "completed" || job.status === "failed") {
      return;
    }
    
    const interval = setInterval(async () => {
      try {
        const updatedJob: Job = await invoke("get_job", { jobId: job.id });
        setJob(updatedJob);
        const paused: boolean = await invoke("is_task_paused");
        setTaskPaused(paused);
        
        if (updatedJob.status === "completed" || updatedJob.status === "failed") {
          clearInterval(interval);
          setBusy("");
          setTaskPaused(false);
          setJob(null);
          const currentDetail = detailRef.current;
          if (currentDetail) {
            await loadNovel(currentDetail.novel.id);
          }
        }
      } catch {
        // Ignore polling errors
      }
    }, 2000);
    
    return () => clearInterval(interval);
  }, [job?.id, job?.status]);

  async function refreshAll() {
    const [novelRows, profileRows, appSettings] = await Promise.all([
      invoke("list_novels"),
      invoke("list_model_profiles"),
      invoke("get_app_settings")
    ]);
    setNovels(novelRows);
    setProfiles(profileRows);
    setSettings(appSettings);
    const savedProfileId = appSettings.selected_profile_id ?? "";
    const savedProfileIsValid = savedProfileId && profileRows.some((p) => p.id === savedProfileId);
    const newSelectedId = savedProfileIsValid ? savedProfileId : profileRows[0]?.id ?? "";
    if (!selectedProfileId || !profileRows.some((p) => p.id === selectedProfileId)) {
      setSelectedProfileId(newSelectedId);
    }
    const activeId = newSelectedId || selectedProfileId;
    const activeProfile = profileRows.find((p) => p.id === activeId);
    if (activeProfile) {
      setProfileDraft({
        id: activeProfile.id,
        name: activeProfile.name,
        provider: activeProfile.provider,
        base_url: activeProfile.base_url,
        model: activeProfile.model,
        temperature: activeProfile.temperature,
        top_p: activeProfile.top_p,
        thinking_mode: activeProfile.thinking_mode as "auto" | "off" | "on",
        api_key: activeProfile.has_api_key ? savedApiKeyMask : "",
      });
    }
    if (novelRows[0]) {
      await loadNovel(novelRows[0].id);
    }
  }

  async function loadNovel(novelId: string) {
    if (processingTaskActive && detail?.novel.id !== novelId) {
      showNotice("当前任务运行或暂停中，不能切换小说。");
      return;
    }
    const next = await invoke("get_novel_detail", { novelId });
    if (next) {
      setDetail(next);
      setSelectedChapterId(next.chapters[0]?.id ?? "");
      setSelectedBatchId(next.batches[0]?.id ?? "");
      await refreshLogs(novelId);
    }
  }

  async function refreshLogs(novelId = detail?.novel.id) {
    try {
      const rows = await invoke("list_ai_logs", { novelId: novelId ?? null });
      setLogs(rows);
    } catch {
      // ignore
    }
  }

  async function clearLogs() {
    const targetText = detail ? `《${detail.novel.title}》相关日志和全局日志` : "所有日志";
    if (!window.confirm(`清空${targetText}？`)) return;
    setBusy("clear-logs");
    setNotice("");
    try {
      await invoke("clear_ai_logs", { novelId: detail?.novel.id ?? null });
      await refreshLogs();
      showNotice("日志已清空。");
    } catch (error) {
      showNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  function isTxtFilePath(filePath: string) {
    return filePath.trim().toLowerCase().endsWith(".txt");
  }

  async function importTxtFile(filePath: string) {
    if (!isTxtFilePath(filePath)) {
      showNotice("当前仅支持导入 TXT 小说文件。");
      return;
    }
    if (importInProgressRef.current) return;
    importInProgressRef.current = true;
    busyRef.current = "import";
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
      importInProgressRef.current = false;
      busyRef.current = "";
      setBusy("");
    }
  }

  async function importTxt() {
    busyRef.current = "import";
    setBusy("import");
    setNotice("");
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "TXT 小说", extensions: ["txt"] }]
      });
      if (typeof selected !== "string") return;
      await importTxtFile(selected);
    } catch (error) {
      showNotice(String(error));
    } finally {
      if (!importInProgressRef.current) {
        busyRef.current = "";
        setBusy("");
      }
    }
  }

  function deleteNovel(novel: Novel) {
    if (processingTaskActive) {
      showNotice("当前任务运行或暂停中，不能删除小说。");
      return;
    }
    setOpenNovelMenuId("");
    setNovelPendingDeletion(novel);
  }

  async function confirmDeleteNovel() {
    const novel = novelPendingDeletion;
    if (!novel) return;
    if (processingTaskActive) {
      setNovelPendingDeletion(null);
      showNotice("当前任务运行或暂停中，不能删除小说。");
      return;
    }
    setBusy("delete-novel");
    setNotice("");
    try {
      await invoke("delete_novel", { novelId: novel.id });
      const remaining = await invoke("list_novels");
      setNovels(remaining);
      setOpenNovelMenuId("");
      if (detail?.novel.id === novel.id) {
        if (remaining[0]) {
          await loadNovel(remaining[0].id);
        } else {
          setDetail(null);
          setSelectedChapterId("");
          setSelectedBatchId("");
        }
      }
      setNovelPendingDeletion(null);
      showNotice(`已删除《${novel.title}》。`);
    } catch (error) {
      showNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function saveProfile() {
    setBusy("profile");
    setNotice("");
    try {
      const input = {
        ...profileDraft,
        id: profileDraft.id && selectedProfileId === profileDraft.id ? profileDraft.id : undefined,
        name: profileDraft.name.trim(),
        provider: profileDraft.provider.trim(),
        base_url: profileDraft.base_url.trim(),
        model: profileDraft.model.trim(),
        api_key: profileDraft.api_key === savedApiKeyMask ? undefined : profileDraft.api_key
      };
      const saved = await invoke("save_model_profile", { input });
      setSelectedProfileId(saved.id);
      setProfileDraft({ ...profileDraft, id: saved.id, api_key: saved.has_api_key ? savedApiKeyMask : "" });
      await persistSelectedProfileId(saved.id);
      await refreshAll();
      showNotice(saved.has_api_key ? "模型配置和 API Key 已保存。" : "模型配置已保存，尚未保存 API Key。");
    } catch (error) {
      showNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  function createNewModelProfile() {
    setSelectedProfileId("");
    setProfileDraft(defaultProfile);
    setOpenModelMenu(false);
    void persistSelectedProfileId("");
    showNotice("已切换为新建模型配置，填写后点击保存。");
  }

  async function deleteSelectedModelProfile() {
    if (processingTaskActive) {
      showNotice("当前任务运行或暂停中，不能删除模型配置。");
      return;
    }
    const profile = profiles.find((item) => item.id === selectedProfileId);
    if (!profile) {
      showNotice("请先选择一个模型配置。");
      return;
    }
    if (!window.confirm(`删除模型配置「${profile.model}」及其保存的 API Key？`)) return;
    setBusy("delete-model");
    setNotice("");
    try {
      await invoke("delete_model_profile", { profileId: profile.id });
      const nextProfiles = await invoke("list_model_profiles");
      setProfiles(nextProfiles);
      const nextSelected = nextProfiles[0]?.id ?? "";
      setSelectedProfileId(nextSelected);
      setOpenModelMenu(false);
      await persistSelectedProfileId(nextSelected);
      if (!nextSelected) setProfileDraft(defaultProfile);
      showNotice(`已删除模型配置「${profile.model}」。`);
    } catch (error) {
      showNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function diagnoseProfile() {
    if (!selectedProfileId) {
      showNotice("请先保存并选择一个模型配置。");
      return;
    }
    setBusy("diagnose");
    setNotice("");
    setModelDiagnosis(null);
    try {
      const result = await invoke("diagnose_model_profile", {
        profileId: selectedProfileId
      });
      setModelDiagnosis(result);
      const label = result.status === "ok" ? "诊断通过" : result.status === "warning" ? "诊断有警告" : "诊断失败";
      showNotice(label);
    } catch (error) {
      showNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function persistSelectedProfileId(profileId: string) {
    try {
      const saved = await invoke("save_selected_profile_id", { profileId: profileId || null });
      setSettings(saved);
    } catch (error) {
      console.error("Failed to persist selected model profile", error);
    }
  }

  function selectModelProfile(profileId: string) {
    setSelectedProfileId(profileId);
    setOpenModelMenu(false);
    void persistSelectedProfileId(profileId);
    const profile = profiles.find((p) => p.id === profileId);
    if (profile) {
      setProfileDraft({
        id: profile.id,
        name: profile.name,
        provider: profile.provider,
        base_url: profile.base_url,
        model: profile.model,
        temperature: profile.temperature,
        top_p: profile.top_p,
        thinking_mode: profile.thinking_mode as "auto" | "off" | "on",
        api_key: profile.has_api_key ? savedApiKeyMask : "",
      });
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
        profileId: selectedProfileId
      });
      setJob(result);
      showNotice(result.message);
    } catch (error) {
      showNotice(String(error));
      setBusy("");
    }
  }

  async function cancelValidation() {
    setNotice("");
    try {
      await invoke("cancel_validation");
      setJob(null);
      setTaskPaused(false);
      setBusy("");
      showNotice("验证已终止", 8000);
    } catch (error) {
      showNotice(String(error));
    }
  }

  async function cancelReview() {
    setNotice("");
    try {
      await invoke("cancel_review");
      setJob(null);
      setTaskPaused(false);
      setBusy("");
      showNotice("审查已终止", 8000);
    } catch (error) {
      showNotice(String(error));
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
        profileId: selectedProfileId
      });
      setJob(result);
      showNotice(result.message);
    } catch (error) {
      showNotice(String(error));
      setBusy("");
    }
  }

  async function pauseReview() {
    setNotice("");
    try {
      await invoke("pause_review");
      showNotice("审查已暂停，点击「继续」恢复", 8000);
    } catch (error) {
      showNotice(String(error));
    }
  }

  async function resumeReview() {
    setNotice("");
    try {
      await invoke("resume_review");
      showNotice("审查已继续", 8000);
    } catch (error) {
      showNotice(String(error));
    }
  }

  async function reviewSingleChapter(chapterId: string) {
    if (!detail || !selectedProfileId) {
      showNotice("请先选择模型配置。");
      return;
    }
    setBusy("review-single");
    setNotice("");
    try {
      await invoke("review_single_chapter", {
        chapterId,
        profileId: selectedProfileId,
      });
      const next = await invoke("get_novel_detail", { novelId: detail.novel.id });
      if (next) {
        setDetail(next);
        setSelectedChapterId(chapterId);
      }
      showNotice("章节审查完成");
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
        outputDir: selected
      });
      showNotice(`已导出：${result.path}`);
    } catch (error) {
      showNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  function displayChapterTitle(chapter: Chapter) {
    const title = chapter.title.replace(/\s+/g, " ").trim();
    return title || `第 ${chapter.index} 章`;
  }

  async function handleUpdateChapter(chapterId: string, title: string) {
    if (!detail) return;
    setBusy("update-chapter");
    try {
      const chapter = detail.chapters.find(c => c.id === chapterId);
      if (chapter) {
        await invoke("update_chapter_text", { chapterId, title, originalText: chapter.original_text });
      }
      await loadNovel(detail.novel.id);
      showNotice("章节标题已更新");
    } catch (error) {
      showNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function handleDeleteChapter(chapterId: string) {
    if (!detail) return;
    setBusy("delete-chapter");
    try {
      await invoke("delete_chapter", { chapterId });
      await loadNovel(detail.novel.id);
      showNotice("章节已删除");
    } catch (error) {
      showNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function handleBatchDeleteChapters(chapterIds: string[]) {
    if (!detail || chapterIds.length === 0) return;
    const novelId = detail.novel.id;
    try {
      await invoke("delete_chapters_batch", { chapterIds });
      const next = await invoke("get_novel_detail", { novelId });
      if (next) {
        setDetail(next);
        setSelectedChapterId(next.chapters[0]?.id ?? "");
        setSelectedBatchId(next.batches[0]?.id ?? "");
      }
      showNotice(`已删除 ${chapterIds.length} 个章节`);
    } catch (error) {
      showNotice(String(error));
    }
  }

  async function handleToggleValidity(chapterId: string, isValid: boolean) {
    if (!detail) return;
    setBusy("toggle-validity");
    try {
      await invoke("toggle_chapter_validity", { chapterId, isValid });
      await loadNovel(detail.novel.id);
      showNotice(isValid ? "章节已标记为有效" : "章节已标记为无效");
    } catch (error) {
      showNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function handleExportDirectory(_chaptersToExport: Chapter[]) {
    setNotice("");
    try {
      const selected = await open({ directory: true, multiple: false });
      if (typeof selected !== "string") return;
      
      const fileName = detail ? `${detail.novel.title}_章节目录.txt` : "章节目录.txt";
      const outputPath = selected + "\\" + fileName;
      
      await invoke("export_chapter_directory", {
        novelId: detail!.novel.id,
        outputPath
      });
      
      showNotice(`章节目录已导出: ${outputPath}`);
    } catch (error) {
      showNotice(String(error));
    }
  }

  async function handleSaveCorrected(chapterId: string, correctedText: string) {
    if (!detail) return;
    try {
      await invoke("update_chapter_corrected_text", {
        chapterId,
        correctedText,
      });
      const next = await invoke("get_novel_detail", { novelId: detail.novel.id });
      if (next) {
        setDetail(next);
        setSelectedChapterId(chapterId);
      }
      showNotice("审查结果已保存");
    } catch (error) {
      showNotice(String(error));
    }
  }

  async function handleRestoreCorrected(chapterId: string) {
    if (!detail) return;
    try {
      await invoke("clear_chapter_review", { chapterId });
      const next = await invoke("get_novel_detail", { novelId: detail.novel.id });
      if (next) {
        setDetail(next);
        setSelectedChapterId(chapterId);
      }
      showNotice("已恢复到原文");
    } catch (error) {
      showNotice(String(error));
    }
  }

  const validChapters = detail?.chapters.filter((c) => c.is_valid) ?? [];
  const invalidChapters = detail?.chapters.filter((c) => !c.is_valid) ?? [];

  const dynamicBatches = useMemo(() => {
    if (!detail) return [];
    const batchSize = settings.chapter_batch_size ?? 30;
    const chapters = detail.chapters;
    const batches: Array<{ id: string; novel_id: string; batch_index: number; label: string; start_chapter: number; end_chapter: number; status: string; created_at: string }> = [];
    for (let i = 0; i < chapters.length; i += batchSize) {
      const end = Math.min(i + batchSize, chapters.length);
      batches.push({
        id: `batch-${i}`,
        novel_id: detail.novel.id,
        batch_index: batches.length + 1,
        label: `第${chapters[i].index}-${chapters[end - 1].index}章`,
        start_chapter: chapters[i].index,
        end_chapter: chapters[end - 1].index,
        status: "pending",
        created_at: "",
      });
    }
    return batches;
  }, [detail, settings.chapter_batch_size]);

  useEffect(() => {
    if (dynamicBatches.length > 0 && !dynamicBatches.some((b) => b.id === selectedBatchId)) {
      setSelectedBatchId(dynamicBatches[0].id);
    }
  }, [dynamicBatches, selectedBatchId]);

  return (
    <div className="app-shell">
      <header className="app-menu">
        <div className="brand">
          <button className="brand-button" onClick={() => setActiveView("workspace")}>
            <BookOpen size={20} />
            <div>
              <strong>NovelProcessor</strong>
              <span>小说处理器</span>
            </div>
          </button>
        </div>
        <div className="app-menu-spacer" />
        <button
          className={`app-menu-item ${activeView === "workspace" ? "active" : ""}`}
          onClick={() => setActiveView("workspace")}
        >
          工作区
        </button>
        {detail && (
          <button
            className={`app-menu-item ${activeView === "compare" ? "active" : ""}`}
            onClick={() => setActiveView("compare")}
          >
            <GitCompareArrows size={14} /> 对比
          </button>
        )}
        <button
          className={`app-menu-item ${activeView === "settings" ? "active" : ""}`}
          onClick={() => setActiveView("settings")}
        >
          设置
        </button>
      </header>

      <aside className="sidebar">
        <div className="side-section">
          <span className="section-label">小说列表</span>
          <button className="primary-action" onClick={importTxt} disabled={!!busy}>
            <FilePlus2 size={16} />导入 TXT
          </button>
          <div className="novel-list">
            {novels.map((novel) => (
              <div className="novel-row" key={novel.id}>
                <button
                  className={`novel-item ${detail?.novel.id === novel.id ? "active" : ""}`}
                  onClick={() => void loadNovel(novel.id)}
                >
                  <span>{novel.title}</span>
                </button>
                <button
                  className="icon-button menu-trigger"
                  onClick={() => setOpenNovelMenuId(openNovelMenuId === novel.id ? "" : novel.id)}
                  disabled={processingTaskActive}
                >
                  <MoreHorizontal size={16} />
                </button>
                {openNovelMenuId === novel.id && (
                  <div className="context-menu">
                    <button onClick={() => deleteNovel(novel)}>
                      <Trash2 size={15} />删除
                    </button>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>

        <div className="side-section">
          <span className="section-label">审查模型</span>
          <ModelProfiles
            profiles={profiles}
            selectedProfileId={selectedProfileId}
            menuOpen={openModelMenu}
            busy={busy}
            processing={processingTaskActive}
            onSelect={selectModelProfile}
            onMenuOpenChange={setOpenModelMenu}
            onDelete={deleteSelectedModelProfile}
          />
        </div>

        <div className="side-section">
          <button
            className={`nav-button ${activeView === "logs" ? "active" : ""}`}
            onClick={() => {
              setActiveView("logs");
              void refreshLogs();
            }}
          >
            <ClipboardList size={16} />
            日志
          </button>
        </div>

        <div className="sidebar-spacer" />
      </aside>

      {activeView === "workspace" && detail && (
        <div className="workspace">
          <div className="topbar">
            <div>
              <h1>{detail.novel.title}</h1>
              <p>
                {detail.chapters.length} 章 · {validChapters.length} 有效 · {invalidChapters.length} 无效
              </p>
            </div>
            <div className="topbar-actions">
              {detail && (busy === "validate" || busy === "review") && (
                <>
                  {busy === "review" && (
                    <button onClick={pauseReview} className="task-control">
                      <Square size={16} />暂停
                    </button>
                  )}
                  {busy === "review" && (
                    <button onClick={resumeReview} className="task-control">
                      <CheckCircle2 size={16} />继续
                    </button>
                  )}
                  <button onClick={busy === "validate" ? cancelValidation : cancelReview} className="task-control-danger">
                    <Square size={16} />终止
                  </button>
                </>
              )}
              {detail && !processingTaskActive && (
                <>
                  <button onClick={runValidation} disabled={!selectedProfileId || !!busy}>
                    {busy === "validate" ? <Loader2 className="spin" size={16} /> : <CheckCircle2 size={16} />}
                    验证章节
                  </button>
                  <button onClick={runReview} disabled={!selectedProfileId || !!busy}>
                    {busy === "review" ? <Loader2 className="spin" size={16} /> : <BookOpen size={16} />}
                    审查内容
                  </button>
                  <button onClick={exportNovel} disabled={!!busy || validChapters.length === 0}>
                    {busy === "export" ? <Loader2 className="spin" size={16} /> : <Download size={16} />}
                    导出
                  </button>
                </>
              )}
            </div>
          </div>

          {notice && <div className="notice">{notice}</div>}

          {job && (
            <div className={`job-strip status-${getStatusTone(job.status)}`}>
              <div className="job-content">
                <span>{job.message}</span>
                {(busy === "validate" || busy === "review") && (
                  <span style={{ marginLeft: "8px", fontSize: "12px", opacity: 0.8 }}>
                    {taskPaused ? "⏸ 已暂停" : "▶ 运行中"}
                  </span>
                )}
                {job.total_chapters > 0 && (
                  <div className="job-progress-row">
                    <div className="job-progress-bar">
                      <div
                        className="job-progress-fill"
                        style={{ width: `${Math.min(100, Math.max(0, (job.current_chapter / job.total_chapters) * 100))}%` }}
                      />
                    </div>
                    <strong>{Math.round((job.current_chapter / job.total_chapters) * 100)}%</strong>
                  </div>
                )}
              </div>
            </div>
          )}

          <BatchPanel
            batches={dynamicBatches}
            selectedBatch={dynamicBatches.find((b) => b.id === selectedBatchId)}
            selectedBatchId={selectedBatchId}
            onSelect={setSelectedBatchId}
          />

          {modelDiagnosis && (
            <div className={`diagnosis-panel diagnosis-top-panel status-container status-${getStatusTone(modelDiagnosis.status)}`}>
              <div className="diagnosis-heading">
                <strong>诊断结果</strong>
                <StatusBadge status={modelDiagnosis.status} label={modelDiagnosis.status} />
                {modelDiagnosis.recommended_thinking_mode && (
                  <span className="diagnosis-recommendation">建议思考模式：{modelDiagnosis.recommended_thinking_mode}</span>
                )}
                <button
                  className="icon-button diagnosis-close"
                  type="button"
                  aria-label="关闭诊断结果"
                  onClick={() => setModelDiagnosis(null)}
                >
                  <X size={16} />
                </button>
              </div>
              <div className="diagnosis-list">
                {modelDiagnosis.checks.map((check) => (
                  <div className="diagnosis-item" key={`${check.name}-${check.message}`}>
                    <StatusBadge status={check.status} label={check.status} showDot={false} />
                    <div>
                      <strong>{check.name}</strong>
                      <p>{check.message}</p>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="content-grid workspace-main-grid">
            <ModelConfig
              draft={profileDraft}
              setDraft={setProfileDraft}
              selectedProfile={selectedProfile}
              selectedProfileId={selectedProfileId}
              suggestions={detectedModelSuggestions}
              suggestionsOpen={openModelSuggestions}
              busy={busy}
              processing={processingTaskActive}
              savedApiKeyMask={savedApiKeyMask}
              onSuggestionsOpenChange={setOpenModelSuggestions}
              onCreate={createNewModelProfile}
              onDiagnose={diagnoseProfile}
              onSave={saveProfile}
            />
            <ChapterList
              chapters={detail.chapters}
              selectedChapterId={selectedChapterId}
              onSelect={setSelectedChapterId}
              displayTitle={displayChapterTitle}
              onUpdateChapter={handleUpdateChapter}
              onDeleteChapter={handleDeleteChapter}
              onBatchDeleteChapters={handleBatchDeleteChapters}
              onToggleValidity={handleToggleValidity}
              onExportDirectory={handleExportDirectory}
            />
          </div>
        </div>
      )}

      {activeView === "workspace" && !detail && (
        <div className="workspace">
          <div className="page-panel" style={{ display: "flex", alignItems: "center", justifyContent: "center" }}>
            <div style={{ textAlign: "center" }}>
              <BookOpen size={48} style={{ color: "#ccc", marginBottom: "16px" }} />
              <h2>欢迎使用 NovelProcessor</h2>
              <p style={{ color: "#666", marginBottom: "16px" }}>请导入 TXT 小说文件开始处理</p>
              <button onClick={importTxt}>
                <FilePlus2 size={16} />导入小说
              </button>
            </div>
          </div>
        </div>
      )}

      {activeView === "settings" && (
        <div className="page-panel">
          <div className="page-heading">
            <h2>设置</h2>
            <button onClick={() => setActiveView("workspace")}><ArrowLeft size={16} />返回</button>
          </div>
          <section className="settings-section">
            <h3>每批次章节数</h3>
            <div className="setting-toggle-row">
              <div className="mode-toggle mode-toggle-three setting-batch-size" role="radiogroup" aria-label="每批次章节数">
                {([30, 50, 100] as const).map((value) => (
                  <button
                    key={value}
                    type="button"
                    className={(settings.chapter_batch_size ?? 30) === value ? "active" : ""}
                    aria-checked={(settings.chapter_batch_size ?? 30) === value}
                    role="radio"
                    disabled={!!busy}
                    onClick={() => {
                      const newSize = value as 30 | 50 | 100;
                      setSettings({ ...settings, chapter_batch_size: newSize });
                      void invoke("save_app_settings", { settings: { ...settings, chapter_batch_size: newSize } });
                    }}
                  >
                    {value} 章
                  </button>
                ))}
              </div>
              <span>默认 30 章。验证按此数量分批发送标题；审查按此数量分组后按并发数均分发送。任务运行中不能修改。</span>
            </div>
          </section>
          <section className="settings-section">
            <h3>审查并发</h3>
            <div className="setting-toggle-row">
              <div className="mode-toggle mode-toggle-six setting-parallelism" role="radiogroup" aria-label="审查并发请求数">
                {([1, 3, 6, 10, 25, 50] as const).map((value) => {
                  const batchSize = settings.chapter_batch_size ?? 30;
                  const maxParallelism = batchSize;
                  const unavailable = value > maxParallelism;
                  return (
                    <button
                      key={value}
                      type="button"
                      className={(settings.review_parallelism ?? 10) === value ? "active" : ""}
                      aria-checked={(settings.review_parallelism ?? 10) === value}
                      role="radio"
                      disabled={!!busy || unavailable}
                      title={unavailable ? `每组 ${batchSize} 章，最高可选并发 ${maxParallelism}` : undefined}
                      onClick={() => {
                        setSettings({ ...settings, review_parallelism: value as 1 | 3 | 6 | 10 | 25 | 50 });
                        void invoke("save_app_settings", { settings: { ...settings, review_parallelism: value } });
                      }}
                    >
                      {value === 1 ? "不并发" : value}
                    </button>
                  );
                })}
              </div>
              <span>默认 10。审查时将每组章节按并发数均分，每份为一个请求。并发越高审查越快，但 API 限流风险越大。</span>
            </div>
          </section>
        </div>
      )}

      {activeView === "logs" && (
        <LogsPage
          logs={logs}
          busy={busy}
          onBack={() => setActiveView("workspace")}
          onClear={clearLogs}
          onRefresh={() => refreshLogs()}
        />
      )}

      {activeView === "compare" && detail && (
        <div className="workspace">
          <CompareView
            chapters={detail.chapters}
            selectedChapterId={selectedChapterId}
            busy={busy}
            originalRef={originalRef}
            correctedRef={correctedRef}
            onSelectChapter={setSelectedChapterId}
            onBack={() => setActiveView("workspace")}
            onExport={exportNovel}
            onSaveCorrected={handleSaveCorrected}
            onRestoreCorrected={handleRestoreCorrected}
            onReviewSingle={reviewSingleChapter}
          />
        </div>
      )}

      {dragActive && (
        <div className="drop-import-overlay">
          <div className="drop-import-card">
            <FilePlus2 size={32} />
            <strong>拖放 TXT 文件到此处</strong>
            <span>松开以导入小说</span>
          </div>
        </div>
      )}

      {novelPendingDeletion && (
        <DeleteNovelDialog
          busy={busy === "delete-novel"}
          novel={novelPendingDeletion}
          onCancel={() => setNovelPendingDeletion(null)}
          onConfirm={confirmDeleteNovel}
        />
      )}
    </div>
  );
}
