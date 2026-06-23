import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "./appStore";

beforeEach(() => {
  useAppStore.getState().reset();
});

describe("appStore", () => {
  it("starts with empty initial state", () => {
    const state = useAppStore.getState();
    expect(state.novels).toEqual([]);
    expect(state.detail).toBeNull();
    expect(state.selectedChapterId).toBe("");
    expect(state.selectedBatchId).toBe("");
    expect(state.profiles).toEqual([]);
    expect(state.selectedProfileId).toBe("");
    expect(state.busy).toBe("");
    expect(state.job).toBeNull();
  });

  it("setNovels updates novels", () => {
    const novels = [
      { id: "1", title: "Test Novel", source_path: "/test.txt", encoding: "utf-8", status: "imported", created_at: "2024-01-01" },
    ];
    useAppStore.getState().setNovels(novels);
    expect(useAppStore.getState().novels).toEqual(novels);
  });

  it("setDetail updates detail", () => {
    const detail = {
      novel: { id: "1", title: "Test", source_path: "", encoding: "", status: "", created_at: "" },
      chapters: [],
      batches: [],
    };
    useAppStore.getState().setDetail(detail);
    expect(useAppStore.getState().detail).toEqual(detail);
  });

  it("setSelectedChapterId updates selectedChapterId", () => {
    useAppStore.getState().setSelectedChapterId("ch-1");
    expect(useAppStore.getState().selectedChapterId).toBe("ch-1");
  });

  it("setSelectedBatchId updates selectedBatchId", () => {
    useAppStore.getState().setSelectedBatchId("batch-1");
    expect(useAppStore.getState().selectedBatchId).toBe("batch-1");
  });

  it("setBusy updates busy", () => {
    useAppStore.getState().setBusy("import");
    expect(useAppStore.getState().busy).toBe("import");
  });

  it("setJob updates job", () => {
    const job = {
      id: "j1",
      novel_id: "1",
      job_type: "validate",
      status: "running",
      current_chapter: 0,
      total_chapters: 10,
      message: "starting",
      created_at: "",
      updated_at: "",
    };
    useAppStore.getState().setJob(job);
    expect(useAppStore.getState().job).toEqual(job);
  });

  it("reset clears all state back to initial", () => {
    useAppStore.getState().setBusy("import");
    useAppStore.getState().setSelectedChapterId("ch-1");
    useAppStore.getState().setSelectedBatchId("batch-1");

    useAppStore.getState().reset();

    const state = useAppStore.getState();
    expect(state.busy).toBe("");
    expect(state.selectedChapterId).toBe("");
    expect(state.selectedBatchId).toBe("");
    expect(state.novels).toEqual([]);
    expect(state.detail).toBeNull();
    expect(state.profiles).toEqual([]);
    expect(state.job).toBeNull();
  });
});
