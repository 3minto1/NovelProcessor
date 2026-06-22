import { memo, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { List, useListRef, type RowComponentProps } from "react-window";
import { Filter, Pencil, Trash2 } from "lucide-react";
import type { Chapter } from "../../types";
import { ScrollablePanel } from "../common/ScrollablePanel";
import { StatusBadge } from "../common/StatusBadge";

export const CHAPTER_VIRTUALIZATION_THRESHOLD = 300;
const CHAPTER_ROW_HEIGHT = 76;

type ChapterListProps = {
  chapters: Chapter[];
  selectedChapterId?: string;
  onSelect: (chapterId: string) => void;
  displayTitle: (chapter: Chapter) => string;
  statusText: Record<string, string>;
  onUpdateChapter?: (chapterId: string, title: string) => void;
  onDeleteChapter?: (chapterId: string) => void;
  onToggleValidity?: (chapterId: string, isValid: boolean) => void;
};

type ChapterRowProps = Pick<ChapterListProps, "chapters" | "selectedChapterId" | "onSelect" | "displayTitle" | "statusText" | "onUpdateChapter" | "onDeleteChapter" | "onToggleValidity">;

type ChapterButtonProps = Omit<ChapterRowProps, "chapters"> & {
  chapter: Chapter;
  buttonRef?: (node: HTMLDivElement | null) => void;
};

const ChapterButton = memo(function ChapterButton({ chapter, selectedChapterId, onSelect, displayTitle, statusText, onUpdateChapter, onDeleteChapter, onToggleValidity, buttonRef }: ChapterButtonProps) {
  const title = `${chapter.index}. ${displayTitle(chapter)}`;
  const [isEditing, setIsEditing] = useState(false);
  const [editTitle, setEditTitle] = useState(chapter.title);

  function handleSave() {
    if (onUpdateChapter) {
      onUpdateChapter(chapter.id, editTitle);
    }
    setIsEditing(false);
  }

  function handleCancel() {
    setEditTitle(chapter.title);
    setIsEditing(false);
  }

  if (isEditing) {
    return (
      <div className="chapter-item active" style={{ flexDirection: "column", gap: "8px", height: "auto", minHeight: "100px", padding: "10px" }}>
        <div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
          <input
            value={editTitle}
            onChange={(e) => setEditTitle(e.target.value)}
            style={{ flex: 1, padding: "6px", fontSize: "14px" }}
            placeholder="章节标题"
            autoFocus
          />
          <button onClick={handleSave} className="action-primary" style={{ fontSize: "12px", padding: "4px 10px", minHeight: "auto" }}>
            保存
          </button>
          <button onClick={handleCancel} style={{ fontSize: "12px", padding: "4px 10px", minHeight: "auto" }}>
            取消
          </button>
        </div>
        <div style={{ display: "flex", gap: "4px" }}>
          <StatusBadge
            status={chapter.validation_status}
            label={statusText[chapter.validation_status] ?? chapter.validation_status}
          />
          <StatusBadge
            status={chapter.review_status}
            label={statusText[chapter.review_status] ?? chapter.review_status}
          />
        </div>
      </div>
    );
  }

  return (
    <div
      ref={buttonRef}
      className={selectedChapterId === chapter.id ? "chapter-item active" : "chapter-item"}
      style={{ cursor: "pointer" }}
    >
      <div onClick={() => onSelect(chapter.id)} style={{ flex: 1, minWidth: 0 }}>
        <span className="chapter-title">{title}</span>
        <span className="chapter-status-row">
          <StatusBadge
            status={chapter.validation_status}
            label={statusText[chapter.validation_status] ?? chapter.validation_status}
          />
          <StatusBadge
            status={chapter.review_status}
            label={statusText[chapter.review_status] ?? chapter.review_status}
          />
        </span>
      </div>
      <div className="chapter-actions">
        {onUpdateChapter && (
          <button
            className="icon-button"
            onClick={(e) => {
              e.stopPropagation();
              setIsEditing(true);
            }}
            title="编辑标题"
          >
            <Pencil size={12} />
          </button>
        )}
        {onToggleValidity && (
          <button
            className="icon-button"
            onClick={(e) => {
              e.stopPropagation();
              onToggleValidity(chapter.id, !chapter.is_valid);
            }}
            title={chapter.is_valid ? "标记为无效" : "标记为有效"}
            style={{ color: chapter.is_valid ? "#e74c3c" : "#27ae60" }}
          >
            {chapter.is_valid ? "无效" : "有效"}
          </button>
        )}
        {onDeleteChapter && (
          <button
            className="icon-button"
            onClick={(e) => {
              e.stopPropagation();
              if (confirm(`确定删除章节 "${chapter.title}"？`)) {
                onDeleteChapter(chapter.id);
              }
            }}
            title="删除章节"
          >
            <Trash2 size={12} />
          </button>
        )}
      </div>
    </div>
  );
});

function ChapterRow({ index, style, ariaAttributes, ...props }: RowComponentProps<ChapterRowProps>) {
  return (
    <div {...ariaAttributes} style={style}>
      <ChapterButton chapter={props.chapters[index]} {...props} />
    </div>
  );
}

function normalizeQuery(value: string) {
  return value.trim().toLowerCase();
}

function isIntegerQuery(value: string) {
  return /^\d+$/.test(value);
}

export const ChapterList = memo(function ChapterList({ chapters, selectedChapterId, onSelect, displayTitle, statusText, onUpdateChapter, onDeleteChapter, onToggleValidity }: ChapterListProps) {
  const listRef = useListRef(null);
  const selectedButtonRef = useRef<HTMLDivElement | null>(null);
  const [jumpQuery, setJumpQuery] = useState("");
  const [showInvalidOnly, setShowInvalidOnly] = useState(false);
  const normalizedJumpQuery = normalizeQuery(jumpQuery);
  
  const filteredChapters = useMemo(() => {
    if (showInvalidOnly) {
      return chapters.filter((c) => !c.is_valid);
    }
    return chapters;
  }, [chapters, showInvalidOnly]);
  
  const visibleChapters = useMemo(() => {
    const query = normalizedJumpQuery;
    if (!query) return filteredChapters;
    const numericQuery = isIntegerQuery(query) ? Number.parseInt(query, 10) : NaN;
    const exactChapter = Number.isFinite(numericQuery)
      ? filteredChapters.find((chapter) => chapter.index === numericQuery)
      : undefined;
    if (exactChapter) return [exactChapter];
    return filteredChapters.filter((chapter) => displayTitle(chapter).toLowerCase().includes(query));
  }, [filteredChapters, displayTitle, normalizedJumpQuery]);
  
  const virtualized = visibleChapters.length >= CHAPTER_VIRTUALIZATION_THRESHOLD;
  const selectedIndex = useMemo(() => visibleChapters.findIndex((chapter) => chapter.id === selectedChapterId), [visibleChapters, selectedChapterId]);
  const rowProps = useMemo(() => ({ chapters: visibleChapters, selectedChapterId, onSelect, displayTitle, statusText, onUpdateChapter, onDeleteChapter, onToggleValidity }), [visibleChapters, selectedChapterId, onSelect, displayTitle, statusText, onUpdateChapter, onDeleteChapter, onToggleValidity]);
  const firstMatch = visibleChapters[0] ?? null;

  function virtualListElement() {
    return listRef.current?.element ?? null;
  }

  function scrollVirtualListToIndex(index: number, align: "center" | "smart" = "smart") {
    if (index < 0) return;
    listRef.current?.scrollToRow({ index, align, behavior: "auto" });
    const element = virtualListElement();
    if (element) {
      const viewportHeight = element.clientHeight || 408;
      const offset =
        align === "center"
          ? Math.max(0, index * CHAPTER_ROW_HEIGHT - Math.max(0, (viewportHeight - CHAPTER_ROW_HEIGHT) / 2))
          : index * CHAPTER_ROW_HEIGHT;
      if (typeof element.scrollTo === "function") {
        element.scrollTo({ top: offset, behavior: "auto" });
      } else {
        element.scrollTop = offset;
      }
      element.dispatchEvent(new Event("scroll", { bubbles: true }));
    }
  }

  function selectFirstMatch() {
    if (!firstMatch) return;
    onSelect(firstMatch.id);
  }

  useLayoutEffect(() => {
    if (!virtualized || selectedIndex < 0) return;
    scrollVirtualListToIndex(selectedIndex, "center");
    const frame = window.requestAnimationFrame(() => {
      scrollVirtualListToIndex(selectedIndex, "center");
    });
    return () => window.cancelAnimationFrame(frame);
  }, [selectedIndex, virtualized]);

  useEffect(() => {
    if (selectedIndex < 0) return;
    if (virtualized) scrollVirtualListToIndex(selectedIndex, "smart");
    else selectedButtonRef.current?.scrollIntoView?.({ block: "nearest" });
  }, [listRef, selectedIndex, virtualized]);

  const invalidCount = chapters.filter((c) => !c.is_valid).length;

  return (
    <section className="panel chapter-list-panel">
      <div className="panel-heading chapter-list-heading">
        <h2>章节 ({visibleChapters.length})</h2>
        <div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
          {invalidCount > 0 && (
            <button
              className={showInvalidOnly ? "action-primary" : ""}
              onClick={() => setShowInvalidOnly(!showInvalidOnly)}
              style={{ fontSize: "12px", padding: "4px 8px", minHeight: "auto" }}
              title={`显示 ${invalidCount} 个无效章节`}
            >
              <Filter size={12} />
              {showInvalidOnly ? "显示全部" : `无效 (${invalidCount})`}
            </button>
          )}
          <input
            aria-label="搜索章节"
            className="chapter-jump-input"
            placeholder="搜索"
            value={jumpQuery}
            onChange={(event) => setJumpQuery(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") selectFirstMatch();
            }}
          />
        </div>
      </div>
      {virtualized ? (
        <List
          className="chapter-list virtual-chapter-list"
          listRef={listRef}
          rowComponent={ChapterRow}
          rowCount={visibleChapters.length}
          rowHeight={CHAPTER_ROW_HEIGHT}
          rowProps={rowProps}
          overscanCount={4}
          defaultHeight={408}
          style={{ height: "100%" }}
        />
      ) : (
        <ScrollablePanel className="chapter-list">
          {visibleChapters.map((chapter) => (
            <ChapterButton
              key={chapter.id}
              chapter={chapter}
              selectedChapterId={selectedChapterId}
              buttonRef={selectedChapterId === chapter.id ? (node) => { selectedButtonRef.current = node; } : undefined}
              onSelect={onSelect}
              displayTitle={displayTitle}
              statusText={statusText}
              onUpdateChapter={onUpdateChapter}
              onDeleteChapter={onDeleteChapter}
              onToggleValidity={onToggleValidity}
            />
          ))}
        </ScrollablePanel>
      )}
    </section>
  );
});
