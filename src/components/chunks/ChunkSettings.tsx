import { useState, useEffect, useRef } from "react";
import { Play, Eye, RefreshCw, CheckCircle, Clock } from "lucide-react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";
import * as api from "../../api/chunk";
import * as sourceApi from "../../api/source";
import { ChunkParams, ChunkPreview, Document, Source, JobEvent } from "../../types";
import { useJobStore } from "../../stores/jobStore";
import { cn } from "../../lib/utils";

export default function ChunkSettings() {
  const { t } = useTranslation();
  const { addJob } = useJobStore();
  const [sources, setSources] = useState<Source[]>([]);
  const [documents, setDocuments] = useState<Document[]>([]);
  const [selectedSourceId, setSelectedSourceId] = useState<number | null>(null);
  const [selectedDocIds, setSelectedDocIds] = useState<Set<number>>(new Set());
  const [rechunk, setRechunk] = useState(false);
  const [hideChunked, setHideChunked] = useState(false);
  const [params, setParams] = useState<ChunkParams>({
    max_len: 2000,
    overlap_len: 200,
    length_unit: "Character",
    preserve_paragraph: true,
    preserve_sentence: true,
    allow_hard_cut: true,
  });
  const [previews, setPreviews] = useState<ChunkPreview[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);
  const currentJobIdRef = useRef<number | null>(null);

  // Job 완료/실패 이벤트 리스닝
  useEffect(() => {
    let unlistenCompleted: UnlistenFn | null = null;
    let unlistenFailed: UnlistenFn | null = null;

    const setupListeners = async () => {
      unlistenCompleted = await listen<JobEvent>("job-completed", (event) => {
        if (event.payload.job_id === currentJobIdRef.current) {
          setSuccessMsg(t("chunks.jobCompleted"));
          currentJobIdRef.current = null;
          loadData();
        }
      });

      unlistenFailed = await listen<JobEvent>("job-failed", (event) => {
        if (event.payload.job_id === currentJobIdRef.current) {
          const errorMsg = event.payload.event_type.type === "failed"
            ? event.payload.event_type.error
            : "Unknown error";
          setError(t("chunks.jobFailed", { error: errorMsg }));
          setSuccessMsg(null);
          currentJobIdRef.current = null;
        }
      });
    };

    setupListeners();

    return () => {
      unlistenCompleted?.();
      unlistenFailed?.();
    };
  }, []);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const [sourcesData, docsData] = await Promise.all([
        sourceApi.listSources(),
        sourceApi.listDocuments({}),
      ]);
      setSources(sourcesData);
      // 폴더별, 파일명순 정렬
      const sorted = [...docsData].sort((a, b) => {
        if (a.source_id !== b.source_id) return a.source_id - b.source_id;
        return a.display_name.localeCompare(b.display_name);
      });
      setDocuments(sorted);
    } catch (e) {
      console.error("데이터 로드 실패:", e);
    }
  };

  // 필터링: 소스 + 청크 상태
  const filteredDocuments = documents.filter((d) => {
    if (selectedSourceId && d.source_id !== selectedSourceId) return false;
    if (hideChunked && d.status === "chunked") return false;
    return true;
  });

  const handleSelectAll = () => {
    const docIds = new Set(filteredDocuments.map((d) => d.id));
    setSelectedDocIds(docIds);
  };

  const handleDeselectAll = () => {
    setSelectedDocIds(new Set());
  };

  const toggleDocSelection = (docId: number) => {
    const newSet = new Set(selectedDocIds);
    if (newSet.has(docId)) {
      newSet.delete(docId);
    } else {
      newSet.add(docId);
    }
    setSelectedDocIds(newSet);
  };

  const handlePreview = async () => {
    const docId = selectedDocIds.size > 0
      ? Array.from(selectedDocIds)[0]
      : filteredDocuments[0]?.id;

    if (!docId) {
      setError(t("chunks.previewHint"));
      return;
    }
    setIsLoading(true);
    setError(null);
    setSuccessMsg(null);
    try {
      const preview = await api.getChunkPreview(docId, params, 5);
      setPreviews(preview);
      if (preview.length === 0) {
        setError(t("chunks.previewEmpty"));
      }
    } catch (e) {
      console.error("미리보기 실패:", e);
      const errorMessage = e instanceof Error ? e.message : String(e);
      setError(`${t("chunks.preview")} ${t("common.error").toLowerCase()}: ${errorMessage}`);
      setPreviews([]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleStartChunking = async () => {
    if (filteredDocuments.length === 0) {
      setError(t("chunks.noDocumentsError"));
      return;
    }
    setIsLoading(true);
    setError(null);
    setSuccessMsg(null);
    try {
      // 선택된 문서 또는 필터된 소스의 문서 처리
      const filter: api.ChunkFilter = {
        rechunk,
      };

      if (selectedDocIds.size > 0) {
        filter.document_ids = Array.from(selectedDocIds);
      } else if (selectedSourceId) {
        filter.source_ids = [selectedSourceId];
      }

      const jobId = await api.startChunkJob(params, filter);
      addJob(jobId, "chunk");
      currentJobIdRef.current = jobId;
      setSuccessMsg(t("chunks.jobStarted"));
      setPreviews([]);
    } catch (e) {
      console.error("청크 생성 시작 실패:", e);
      const errorMessage = e instanceof Error ? e.message : String(e);
      setError(t("chunks.jobFailed", { error: errorMessage }));
    } finally {
      setIsLoading(false);
    }
  };

  const getSourceName = (sourceId: number) => {
    const source = sources.find((s) => s.id === sourceId);
    return source?.display_name || source?.path_or_key || `Source #${sourceId}`;
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">{t("chunks.title")}</h1>
        <div className="flex gap-2">
          <button
            onClick={handlePreview}
            disabled={isLoading}
            className="flex items-center gap-2 rounded-md bg-secondary px-3 py-2 text-sm hover:bg-secondary/80 disabled:opacity-50"
          >
            <Eye className="h-4 w-4" />
            {t("chunks.preview")}
          </button>
          <button
            onClick={handleStartChunking}
            disabled={isLoading}
            className="flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
          >
            <Play className="h-4 w-4" />
            {t("chunks.startChunking")}
          </button>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        {/* 문서 선택 패널 */}
        <div className="space-y-4 rounded-lg border border-border bg-card p-6">
          <div className="flex items-center justify-between">
            <h2 className="font-medium">{t("chunks.documentSelection")}</h2>
            <button
              onClick={loadData}
              className="p-1 rounded hover:bg-secondary"
              title={t("common.refresh")}
            >
              <RefreshCw className="h-4 w-4" />
            </button>
          </div>

          {/* 소스 필터 */}
          <div className="space-y-2">
            <label className="text-sm text-muted-foreground">{t("chunks.folderFilter")}</label>
            <select
              value={selectedSourceId || ""}
              onChange={(e) => {
                setSelectedSourceId(e.target.value ? Number(e.target.value) : null);
                setSelectedDocIds(new Set());
              }}
              className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
            >
              <option value="">{t("chunks.allSources")}</option>
              {sources.map((source) => (
                <option key={source.id} value={source.id}>
                  {source.display_name || source.path_or_key}
                </option>
              ))}
            </select>
          </div>

          {/* 문서 목록 */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-sm text-muted-foreground">
                {t("chunks.documentsSelected", { selected: selectedDocIds.size, total: filteredDocuments.length })}
              </label>
              <div className="flex gap-2">
                <button
                  onClick={handleSelectAll}
                  className="text-xs text-primary hover:underline"
                >
                  {t("common.selectAll")}
                </button>
                <button
                  onClick={handleDeselectAll}
                  className="text-xs text-muted-foreground hover:underline"
                >
                  {t("common.deselectAll")}
                </button>
              </div>
            </div>

            {/* 청크된 문서 숨기기 옵션 */}
            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={hideChunked}
                onChange={(e) => {
                  setHideChunked(e.target.checked);
                  setSelectedDocIds(new Set());
                }}
                className="h-4 w-4 rounded"
              />
              <span className="text-muted-foreground">{t("chunks.hideChunked")}</span>
            </label>

            <div className="max-h-[300px] overflow-y-auto border border-border rounded-md">
              {filteredDocuments.length === 0 ? (
                <p className="p-3 text-sm text-muted-foreground">
                  {hideChunked ? t("chunks.noUnprocessed") : t("chunks.noDocuments")}
                </p>
              ) : (
                filteredDocuments.map((doc) => (
                  <label
                    key={doc.id}
                    className={cn(
                      "flex items-center gap-2 px-3 py-2 cursor-pointer hover:bg-secondary border-b border-border last:border-0",
                      selectedDocIds.has(doc.id) && "bg-primary/5"
                    )}
                  >
                    <input
                      type="checkbox"
                      checked={selectedDocIds.has(doc.id)}
                      onChange={() => toggleDocSelection(doc.id)}
                      className="h-4 w-4 rounded"
                    />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <p className="text-sm truncate flex-1">{doc.display_name}</p>
                        {doc.status === "chunked" ? (
                          <span className="flex items-center gap-1 rounded-full bg-green-100 px-2 py-0.5 text-xs text-green-700 dark:bg-green-900 dark:text-green-300">
                            <CheckCircle className="h-3 w-3" />
                            {t("sources.status.chunked")}
                          </span>
                        ) : (
                          <span className="flex items-center gap-1 rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-600 dark:bg-gray-800 dark:text-gray-400">
                            <Clock className="h-3 w-3" />
                            {t("sources.status.unprocessed")}
                          </span>
                        )}
                      </div>
                      {!selectedSourceId && (
                        <p className="text-xs text-muted-foreground mt-0.5">
                          {getSourceName(doc.source_id)}
                        </p>
                      )}
                    </div>
                  </label>
                ))
              )}
            </div>
          </div>

          {/* 재청크 옵션 */}
          <label className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={rechunk}
              onChange={(e) => setRechunk(e.target.checked)}
              className="h-4 w-4 rounded"
            />
            <span className="text-sm">{t("chunks.rechunk")}</span>
          </label>
        </div>

        {/* 설정 패널 */}
        <div className="space-y-6 rounded-lg border border-border bg-card p-6">
          <h2 className="font-medium">{t("chunks.parameters")}</h2>

          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm text-muted-foreground">{t("chunks.maxLength")}</label>
                <input
                  type="number"
                  value={params.max_len}
                  onChange={(e) =>
                    setParams({ ...params, max_len: Number(e.target.value) })
                  }
                  min={100}
                  className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm text-muted-foreground">{t("chunks.overlapLength")}</label>
                <input
                  type="number"
                  value={params.overlap_len}
                  onChange={(e) =>
                    setParams({ ...params, overlap_len: Number(e.target.value) })
                  }
                  min={0}
                  className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                />
              </div>
            </div>

            <div className="space-y-2">
              <label className="text-sm text-muted-foreground">{t("chunks.lengthUnit")}</label>
              <div className="flex gap-4">
                <label className="flex items-center gap-2">
                  <input
                    type="radio"
                    checked={params.length_unit === "Character"}
                    onChange={() => setParams({ ...params, length_unit: "Character" })}
                    className="h-4 w-4"
                  />
                  <span className="text-sm">{t("chunks.character")}</span>
                </label>
                <label className="flex items-center gap-2">
                  <input
                    type="radio"
                    checked={params.length_unit === "Token"}
                    onChange={() => setParams({ ...params, length_unit: "Token" })}
                    className="h-4 w-4"
                  />
                  <span className="text-sm">{t("chunks.token")}</span>
                </label>
              </div>
            </div>

            <div className="space-y-2">
              <label className="text-sm text-muted-foreground">{t("chunks.boundaryRules")}</label>
              <div className="space-y-2">
                <label className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={params.preserve_paragraph}
                    onChange={(e) =>
                      setParams({ ...params, preserve_paragraph: e.target.checked })
                    }
                    className="h-4 w-4 rounded"
                  />
                  <span className="text-sm">{t("chunks.preserveParagraph")}</span>
                </label>
                <label className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={params.preserve_sentence}
                    onChange={(e) =>
                      setParams({ ...params, preserve_sentence: e.target.checked })
                    }
                    className="h-4 w-4 rounded"
                  />
                  <span className="text-sm">{t("chunks.preserveSentence")}</span>
                </label>
                <label className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={params.allow_hard_cut}
                    onChange={(e) =>
                      setParams({ ...params, allow_hard_cut: e.target.checked })
                    }
                    className="h-4 w-4 rounded"
                  />
                  <span className="text-sm">{t("chunks.allowHardCut")}</span>
                </label>
              </div>
            </div>

            <div className="rounded-md bg-muted p-3">
              <p className="text-xs text-muted-foreground">
                <strong>Stride</strong> = max_len - overlap_len ={" "}
                <strong>{params.max_len - params.overlap_len}</strong>
              </p>
            </div>
          </div>
        </div>

        {/* 미리보기 패널 */}
        <div className="space-y-4 rounded-lg border border-border bg-card p-6">
          <h2 className="font-medium">{t("chunks.previewPanel")}</h2>

          {error && (
            <div className="rounded-md bg-red-50 dark:bg-red-950 border border-red-200 dark:border-red-800 p-3">
              <p className="text-sm text-red-700 dark:text-red-300">{error}</p>
            </div>
          )}

          {successMsg && (
            <div className="rounded-md bg-green-50 dark:bg-green-950 border border-green-200 dark:border-green-800 p-3">
              <p className="text-sm text-green-700 dark:text-green-300">{successMsg}</p>
            </div>
          )}

          {!error && !successMsg && previews.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              {t("chunks.previewHint")}
            </p>
          ) : previews.length > 0 ? (
            <div className="space-y-3 max-h-[500px] overflow-y-auto">
              {previews.map((chunk) => (
                <div
                  key={chunk.index}
                  className={cn(
                    "rounded-md border p-3",
                    chunk.is_hard_cut
                      ? "border-yellow-500 bg-yellow-50 dark:bg-yellow-950"
                      : "border-border"
                  )}
                >
                  <div className="mb-2 flex items-center justify-between text-xs">
                    <span className="font-medium">{t("chunks.chunk")} #{chunk.index + 1}</span>
                    <div className="flex gap-2 text-muted-foreground">
                      <span>{chunk.char_count} {t("common.characters")}</span>
                      <span>~{chunk.token_est} {t("common.tokens")}</span>
                      {chunk.is_hard_cut && (
                        <span className="text-yellow-600">{t("chunks.hardCut")}</span>
                      )}
                    </div>
                  </div>
                  <p className="whitespace-pre-wrap text-sm font-mono line-clamp-4">
                    {chunk.text}
                  </p>
                </div>
              ))}
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
}
