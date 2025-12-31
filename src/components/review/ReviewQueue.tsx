import { useEffect, useCallback, useState } from "react";
import { CheckCircle, XCircle, SkipForward, ChevronLeft, ChevronRight, Merge, Scissors, Edit3, AlertTriangle, Info, AlertCircle, Filter, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useReviewStore } from "../../stores/reviewStore";
import { useKeyboard } from "../../hooks/useKeyboard";
import { formatNumber } from "../../lib/utils";
import { mergeChunks, splitChunk } from "../../api/chunk";
import { saveChunkEdit, checkChunkQuality, QualityCheckResult, bulkSetReviewStatus, BulkReviewFilter } from "../../api/review";
import * as sourceApi from "../../api/source";
import { Source } from "../../types";

export default function ReviewQueue() {
  const { t } = useTranslation();
  const {
    queue,
    currentChunk,
    currentIndex,
    isLoading,
    filters,
    loadQueue,
    setStatus,
    setFilters,
    next,
    prev,
  } = useReviewStore();

  const [isEditing, setIsEditing] = useState(false);
  const [editText, setEditText] = useState("");
  const [showSplitDialog, setShowSplitDialog] = useState(false);
  const [splitPosition, setSplitPosition] = useState(0);
  const [isProcessing, setIsProcessing] = useState(false);
  const [qualityResult, setQualityResult] = useState<QualityCheckResult | null>(null);
  const [showQualityDetails, setShowQualityDetails] = useState(false);
  const [sources, setSources] = useState<Source[]>([]);
  const [showBulkActions, setShowBulkActions] = useState(false);
  const [bulkActionResult, setBulkActionResult] = useState<string | null>(null);

  // 소스 목록 로딩
  useEffect(() => {
    sourceApi.listSources().then(setSources).catch(console.error);
  }, []);

  useEffect(() => {
    loadQueue(filters);
  }, []);

  useEffect(() => {
    if (currentChunk) {
      setEditText(currentChunk.edited_text || currentChunk.original_text);
      setIsEditing(false);
      setShowQualityDetails(false);
      // 품질 검사 실행
      checkChunkQuality(currentChunk.chunk.id)
        .then(setQualityResult)
        .catch(() => setQualityResult(null));
    }
  }, [currentChunk]);

  // 키보드 단축키
  const handleApprove = useCallback(async () => {
    if (!currentChunk) return;
    await setStatus(currentChunk.chunk.id, "approved");
    await next();
  }, [currentChunk, setStatus, next]);

  const handleSkip = useCallback(async () => {
    if (!currentChunk) return;
    await setStatus(currentChunk.chunk.id, "skipped");
    await next();
  }, [currentChunk, setStatus, next]);

  const handleReject = useCallback(async () => {
    if (!currentChunk) return;
    await setStatus(currentChunk.chunk.id, "rejected");
    await next();
  }, [currentChunk, setStatus, next]);

  // 편집 저장
  const handleSaveEdit = useCallback(async () => {
    if (!currentChunk) return;
    setIsProcessing(true);
    try {
      await saveChunkEdit(currentChunk.chunk.id, editText);
      setIsEditing(false);
      await loadQueue();
    } catch (e) {
      console.error("편집 저장 실패:", e);
    } finally {
      setIsProcessing(false);
    }
  }, [currentChunk, editText, loadQueue]);

  // 다음 청크와 병합
  const handleMergeWithNext = useCallback(async () => {
    if (!currentChunk || !currentChunk.next_chunk_id) return;
    if (!confirm(t("review.mergeConfirm"))) return;

    setIsProcessing(true);
    try {
      await mergeChunks(currentChunk.chunk.id, currentChunk.next_chunk_id);
      await loadQueue();
    } catch (e) {
      console.error("병합 실패:", e);
      alert("병합 실패: " + (e instanceof Error ? e.message : String(e)));
    } finally {
      setIsProcessing(false);
    }
  }, [currentChunk, loadQueue]);

  // 청크 분할
  const handleSplit = useCallback(async () => {
    if (!currentChunk || splitPosition <= 0) return;

    setIsProcessing(true);
    try {
      await splitChunk(currentChunk.chunk.id, splitPosition);
      setShowSplitDialog(false);
      await loadQueue();
    } catch (e) {
      console.error("분할 실패:", e);
      alert("분할 실패: " + (e instanceof Error ? e.message : String(e)));
    } finally {
      setIsProcessing(false);
    }
  }, [currentChunk, splitPosition, loadQueue]);

  // 필터 변경 핸들러
  const handleFilterChange = useCallback((newFilters: Partial<typeof filters>) => {
    const updated = { ...filters, ...newFilters };
    setFilters(updated);
    loadQueue(updated);
    setBulkActionResult(null);
  }, [filters, setFilters, loadQueue]);

  // 일괄 상태 변경
  const handleBulkAction = useCallback(async (status: "approved" | "skipped" | "rejected") => {
    const statusLabel = status === "approved" ? t("review.approvedStatus") : status === "skipped" ? t("review.skipped") : t("review.rejected");
    if (!confirm(t("review.bulkConfirm", { status: statusLabel }))) {
      return;
    }

    setIsProcessing(true);
    setBulkActionResult(null);
    try {
      const bulkFilter: BulkReviewFilter = {
        current_status: filters.status,
      };
      if (filters.source_id) {
        bulkFilter.source_ids = [filters.source_id];
      }
      if (filters.document_id) {
        bulkFilter.document_ids = [filters.document_id];
      }

      const count = await bulkSetReviewStatus(bulkFilter, status);
      const statusLabel = status === "approved" ? t("review.approvedStatus") : status === "skipped" ? t("review.skipped") : t("review.rejected");
      setBulkActionResult(t("review.bulkSuccess", { count, status: statusLabel }));
      await loadQueue(filters);
    } catch (e) {
      console.error("Bulk action failed:", e);
      setBulkActionResult(t("review.bulkFailed", { error: e instanceof Error ? e.message : String(e) }));
    } finally {
      setIsProcessing(false);
      setShowBulkActions(false);
    }
  }, [filters, loadQueue]);

  // 편집 모드 토글
  const handleToggleEdit = useCallback(() => {
    if (isEditing) {
      // 취소 시 원래 텍스트로 복원
      setEditText(currentChunk?.edited_text || currentChunk?.original_text || "");
    }
    setIsEditing(!isEditing);
  }, [isEditing, currentChunk]);

  useKeyboard({
    Enter: handleApprove,
    s: handleSkip,
    r: handleReject,
    b: prev,
    n: next,
    e: handleToggleEdit,
  });

  if (isLoading && !queue) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-muted-foreground">{t("common.loading")}</p>
      </div>
    );
  }

  if (!queue || queue.chunks.length === 0) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-4">
        <p className="text-muted-foreground">{t("review.noChunks")}</p>
        <p className="text-sm text-muted-foreground">
          {t("review.noChunksHint")}
        </p>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      {/* 헤더 */}
      <div className="flex items-center justify-between border-b border-border pb-4">
        <div className="flex items-center gap-6">
          <div>
            <h1 className="text-2xl font-bold">{t("review.title")}</h1>
            <p className="text-sm text-muted-foreground">
              {formatNumber(queue.approved)} / {formatNumber(queue.total)} {t("review.approved")}
              ({queue.total > 0 ? Math.round((queue.approved / queue.total) * 100) : 0}%)
            </p>
          </div>

          {/* 필터 */}
          <div className="flex items-center gap-3">
            {/* 상태 필터 */}
            <select
              value={filters.status || ""}
              onChange={(e) => handleFilterChange({ status: e.target.value || undefined })}
              className="rounded-md border border-input bg-background px-3 py-1.5 text-sm"
            >
              <option value="">{t("review.allStatus")}</option>
              <option value="pending">{t("review.pending")}</option>
              <option value="approved">{t("review.approvedStatus")}</option>
              <option value="skipped">{t("review.skipped")}</option>
              <option value="rejected">{t("review.rejected")}</option>
            </select>

            {/* 소스 필터 */}
            <select
              value={filters.source_id || ""}
              onChange={(e) => handleFilterChange({ source_id: e.target.value ? Number(e.target.value) : undefined })}
              className="rounded-md border border-input bg-background px-3 py-1.5 text-sm"
            >
              <option value="">{t("review.allSources")}</option>
              {sources.map((source) => (
                <option key={source.id} value={source.id}>
                  {source.display_name || source.path_or_key}
                </option>
              ))}
            </select>

            {/* 새로고침 */}
            <button
              onClick={() => loadQueue(filters)}
              disabled={isLoading}
              className="p-1.5 rounded hover:bg-secondary"
              title={t("common.refresh")}
            >
              <RefreshCw className={`h-4 w-4 ${isLoading ? "animate-spin" : ""}`} />
            </button>

            {/* 일괄 처리 */}
            <div className="relative">
              <button
                onClick={() => setShowBulkActions(!showBulkActions)}
                disabled={isProcessing || queue.chunks.length === 0}
                className="flex items-center gap-1.5 rounded-md border border-input bg-background px-3 py-1.5 text-sm hover:bg-secondary disabled:opacity-50"
              >
                <Filter className="h-4 w-4" />
                {t("review.bulkActions")}
              </button>
              {showBulkActions && (
                <div className="absolute top-full left-0 mt-1 z-10 w-40 rounded-md border border-border bg-background shadow-lg">
                  <button
                    onClick={() => handleBulkAction("approved")}
                    className="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-secondary text-green-600"
                  >
                    <CheckCircle className="h-4 w-4" />
                    {t("review.approveAll")}
                  </button>
                  <button
                    onClick={() => handleBulkAction("skipped")}
                    className="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-secondary"
                  >
                    <SkipForward className="h-4 w-4" />
                    {t("review.skipAll")}
                  </button>
                  <button
                    onClick={() => handleBulkAction("rejected")}
                    className="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-secondary text-red-600"
                  >
                    <XCircle className="h-4 w-4" />
                    {t("review.rejectAll")}
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={prev}
            disabled={currentIndex === 0}
            className="rounded-md p-2 hover:bg-secondary disabled:opacity-50"
          >
            <ChevronLeft className="h-5 w-5" />
          </button>
          <span className="text-sm">
            {currentIndex + 1} / {queue.chunks.length}
          </span>
          <button
            onClick={next}
            disabled={currentIndex >= queue.chunks.length - 1}
            className="rounded-md p-2 hover:bg-secondary disabled:opacity-50"
          >
            <ChevronRight className="h-5 w-5" />
          </button>
        </div>
      </div>

      {/* 일괄 처리 결과 메시지 */}
      {bulkActionResult && (
        <div className={`mt-2 rounded-md p-3 text-sm ${
          bulkActionResult.includes("실패")
            ? "bg-red-50 text-red-700 dark:bg-red-950 dark:text-red-300"
            : "bg-green-50 text-green-700 dark:bg-green-950 dark:text-green-300"
        }`}>
          {bulkActionResult}
        </div>
      )}

      {/* 메인 콘텐츠 */}
      {currentChunk && (
        <div className="flex flex-1 flex-col gap-4 py-4 overflow-hidden">
          {/* 메타 정보 */}
          <div className="flex items-center justify-between text-sm">
            <div className="flex items-center gap-4">
              <span className="text-muted-foreground">
                {t("review.document")}: <span className="text-foreground">{currentChunk.document_name}</span>
              </span>
              <span className="text-muted-foreground">
                {t("review.chunkIndex")}: <span className="text-foreground">#{currentChunk.chunk.chunk_index + 1}</span>
              </span>
            </div>
            <div className="flex items-center gap-2">
              <div className="flex items-center gap-4 text-muted-foreground mr-4">
                <span>{currentChunk.chunk.char_count} {t("common.characters")}</span>
                <span>~{currentChunk.chunk.token_est} {t("common.tokens")}</span>
                {currentChunk.chunk.is_hard_cut && (
                  <span className="rounded bg-yellow-100 px-2 py-0.5 text-xs text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300">
                    {t("chunks.hardCut")}
                  </span>
                )}
                {/* 품질 배지 */}
                {qualityResult && qualityResult.warnings.length > 0 && (
                  <button
                    onClick={() => setShowQualityDetails(!showQualityDetails)}
                    className={`flex items-center gap-1 rounded px-2 py-0.5 text-xs ${
                      qualityResult.warnings.some(w => w.severity === "Error")
                        ? "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300"
                        : qualityResult.warnings.some(w => w.severity === "Warning")
                        ? "bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300"
                        : "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300"
                    }`}
                    title={t("review.qualityWarnings")}
                  >
                    {qualityResult.warnings.some(w => w.severity === "Error") ? (
                      <AlertCircle className="h-3 w-3" />
                    ) : qualityResult.warnings.some(w => w.severity === "Warning") ? (
                      <AlertTriangle className="h-3 w-3" />
                    ) : (
                      <Info className="h-3 w-3" />
                    )}
                    {qualityResult.warnings.length}
                  </button>
                )}
              </div>
              <button
                onClick={handleToggleEdit}
                disabled={isProcessing}
                className={`p-1.5 rounded hover:bg-secondary ${isEditing ? "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300" : ""}`}
                title={`${t("common.edit")} (E)`}
              >
                <Edit3 className="h-4 w-4" />
              </button>
              <button
                onClick={() => {
                  setSplitPosition(Math.floor(currentChunk.chunk.char_count / 2));
                  setShowSplitDialog(true);
                }}
                disabled={isProcessing}
                className="p-1.5 rounded hover:bg-secondary"
                title={t("review.split")}
              >
                <Scissors className="h-4 w-4" />
              </button>
              {currentChunk.next_chunk_id && (
                <button
                  onClick={handleMergeWithNext}
                  disabled={isProcessing}
                  className="p-1.5 rounded hover:bg-secondary"
                  title={t("review.mergeWithNext")}
                >
                  <Merge className="h-4 w-4" />
                </button>
              )}
            </div>
          </div>

          {/* 품질 경고 패널 */}
          {showQualityDetails && qualityResult && qualityResult.warnings.length > 0 && (
            <div className="rounded-lg border border-border bg-muted p-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm font-medium">{t("review.qualityWarnings")}</span>
                <span className="text-xs text-muted-foreground">
                  {t("review.score")}: {Math.round(qualityResult.score * 100)}%
                </span>
              </div>
              <ul className="space-y-1">
                {qualityResult.warnings.map((warning, idx) => (
                  <li key={idx} className="flex items-center gap-2 text-sm">
                    {warning.severity === "Error" ? (
                      <AlertCircle className="h-4 w-4 text-red-500" />
                    ) : warning.severity === "Warning" ? (
                      <AlertTriangle className="h-4 w-4 text-orange-500" />
                    ) : (
                      <Info className="h-4 w-4 text-blue-500" />
                    )}
                    <span className="text-muted-foreground">{warning.message}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {/* 에디터 */}
          <div className="flex-1 overflow-auto">
            <div className="rounded-lg border border-border bg-card p-6 h-full">
              {isEditing ? (
                <div className="flex flex-col h-full gap-2">
                  <textarea
                    value={editText}
                    onChange={(e) => setEditText(e.target.value)}
                    className="flex-1 w-full resize-none bg-transparent font-mono text-sm leading-relaxed focus:outline-none"
                    autoFocus
                  />
                  <div className="flex justify-end gap-2 pt-2 border-t border-border">
                    <button
                      onClick={handleToggleEdit}
                      className="px-3 py-1.5 text-sm rounded hover:bg-secondary"
                    >
                      {t("common.cancel")}
                    </button>
                    <button
                      onClick={handleSaveEdit}
                      disabled={isProcessing}
                      className="px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
                    >
                      {t("common.save")}
                    </button>
                  </div>
                </div>
              ) : (
                <p className="whitespace-pre-wrap font-mono text-sm leading-relaxed">
                  {currentChunk.edited_text || currentChunk.original_text}
                </p>
              )}
            </div>
          </div>

          {/* 오버랩 정보 */}
          <div className="flex items-center gap-4 text-xs text-muted-foreground">
            <span>{t("review.overlapPrev")}: {currentChunk.chunk.overlap_prev}</span>
            <span>{t("review.overlapNext")}: {currentChunk.chunk.overlap_next}</span>
          </div>
        </div>
      )}

      {/* 액션 바 */}
      <div className="flex items-center justify-between border-t border-border pt-4">
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <span>{t("review.shortcuts")}:</span>
          <kbd>Enter</kbd> {t("review.approve")}
          <kbd>S</kbd> {t("review.skip")}
          <kbd>R</kbd> {t("review.reject")}
          <kbd>B</kbd> {t("review.prev")}
          <kbd>N</kbd> {t("review.next")}
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleSkip}
            className="flex items-center gap-2 rounded-md bg-secondary px-4 py-2 text-sm hover:bg-secondary/80"
          >
            <SkipForward className="h-4 w-4" />
            {t("review.skip")} (S)
          </button>
          <button
            onClick={handleReject}
            className="flex items-center gap-2 rounded-md bg-destructive px-4 py-2 text-sm text-destructive-foreground hover:bg-destructive/90"
          >
            <XCircle className="h-4 w-4" />
            {t("review.reject")} (R)
          </button>
          <button
            onClick={handleApprove}
            className="flex items-center gap-2 rounded-md bg-green-600 px-4 py-2 text-sm text-white hover:bg-green-700"
          >
            <CheckCircle className="h-4 w-4" />
            {t("review.approve")} (Enter)
          </button>
        </div>
      </div>

      {/* 분할 다이얼로그 */}
      {showSplitDialog && currentChunk && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-background rounded-lg shadow-lg max-w-lg w-full mx-4 p-6">
            <h3 className="font-medium text-lg mb-4">{t("review.splitChunk")}</h3>
            <div className="space-y-4">
              <div>
                <label className="text-sm text-muted-foreground block mb-2">
                  {t("review.splitPosition")} (1 ~ {currentChunk.chunk.char_count - 1})
                </label>
                <input
                  type="range"
                  min={1}
                  max={currentChunk.chunk.char_count - 1}
                  value={splitPosition}
                  onChange={(e) => setSplitPosition(Number(e.target.value))}
                  className="w-full"
                />
                <div className="flex justify-between text-sm text-muted-foreground mt-1">
                  <span>{t("review.firstChunk")}: {splitPosition} {t("common.characters")}</span>
                  <span>{t("review.secondChunk")}: {currentChunk.chunk.char_count - splitPosition} {t("common.characters")}</span>
                </div>
              </div>
              <div className="text-sm bg-muted rounded p-3 max-h-40 overflow-auto">
                <div className="flex gap-2">
                  <div className="flex-1 border-r border-border pr-2">
                    <p className="text-xs text-muted-foreground mb-1">{t("review.firstChunk")}</p>
                    <p className="font-mono text-xs whitespace-pre-wrap">
                      {(currentChunk.edited_text || currentChunk.original_text).slice(0, splitPosition)}
                      <span className="bg-yellow-200 dark:bg-yellow-900">|</span>
                    </p>
                  </div>
                  <div className="flex-1 pl-2">
                    <p className="text-xs text-muted-foreground mb-1">{t("review.secondChunk")}</p>
                    <p className="font-mono text-xs whitespace-pre-wrap">
                      {(currentChunk.edited_text || currentChunk.original_text).slice(splitPosition)}
                    </p>
                  </div>
                </div>
              </div>
            </div>
            <div className="flex justify-end gap-2 mt-6">
              <button
                onClick={() => setShowSplitDialog(false)}
                className="px-4 py-2 text-sm rounded hover:bg-secondary"
              >
                {t("common.cancel")}
              </button>
              <button
                onClick={handleSplit}
                disabled={isProcessing}
                className="px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
              >
                {t("review.split")}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
