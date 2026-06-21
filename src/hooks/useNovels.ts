import { useCallback } from "react";
import { useAppStore } from "../store/appStore";
import { invokeCommand } from "../tauriApi";
import type { NovelDetail } from "../types";

export function useNovels() {
  const {
    detail,
    setDetail,
    selectedChapterId,
    setSelectedChapterId,
    selectedBatchId,
    setSelectedBatchId,
  } = useAppStore();

  const loadNovel = useCallback(
    async (novelId: string) => {
      const next: NovelDetail = await invokeCommand("get_novel_detail", {
        novelId,
      });
      setDetail(next);
      setSelectedChapterId(next.chapters[0]?.id ?? "");
      setSelectedBatchId(next.batches[0]?.id ?? "");
    },
    [setDetail, setSelectedChapterId, setSelectedBatchId]
  );

  const refreshNovel = useCallback(async () => {
    if (!detail) return;
    await loadNovel(detail.novel.id);
  }, [detail, loadNovel]);

  return {
    detail,
    selectedChapterId,
    setSelectedChapterId,
    selectedBatchId,
    setSelectedBatchId,
    loadNovel,
    refreshNovel,
    selectedChapter: detail?.chapters.find((c) => c.id === selectedChapterId),
    selectedBatch: detail?.batches.find((b) => b.id === selectedBatchId),
  };
}
