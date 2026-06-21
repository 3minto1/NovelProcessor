import { useMemo } from "react";
import { useAppStore } from "../store/appStore";

export function useNovels() {
  const novels = useAppStore((state) => state.novels);
  const setNovels = useAppStore((state) => state.setNovels);
  const detail = useAppStore((state) => state.detail);
  const setDetail = useAppStore((state) => state.setDetail);
  const selectedChapterId = useAppStore((state) => state.selectedChapterId);
  const setSelectedChapterId = useAppStore((state) => state.setSelectedChapterId);
  const selectedBatchId = useAppStore((state) => state.selectedBatchId);
  const setSelectedBatchId = useAppStore((state) => state.setSelectedBatchId);

  const selectedChapter = useMemo(
    () => detail?.chapters.find((chapter) => chapter.id === selectedChapterId) ?? detail?.chapters[0],
    [detail, selectedChapterId]
  );
  const selectedBatch = useMemo(
    () => detail?.batches.find((batch) => batch.id === selectedBatchId) ?? detail?.batches[0],
    [detail, selectedBatchId]
  );

  return {
    novels,
    setNovels,
    detail,
    setDetail,
    selectedChapterId,
    setSelectedChapterId,
    selectedBatchId,
    setSelectedBatchId,
    selectedChapter,
    selectedBatch,
  };
}
