import { useState, useEffect } from "react";
import {
  FolderPlus,
  FilePlus,
  FileText,
  RefreshCw,
  Trash2,
  ChevronDown,
  ChevronRight,
  Folder,
  File,
  Edit3,
  Eye,
  AlertTriangle,
} from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { useTranslation } from "react-i18next";
import * as api from "../../api/source";
import { Document, Source } from "../../types";
import { formatBytes, formatDate, cn } from "../../lib/utils";
import { ConfirmDialog } from "../common/ConfirmDialog";
import { useConfirmDialog, confirmPresets } from "../../hooks/useConfirmDialog";
import { ChangeDetection } from "./ChangeDetection";

export default function Sources() {
  const { t } = useTranslation();
  const [sources, setSources] = useState<Source[]>([]);
  const [documents, setDocuments] = useState<Document[]>([]);
  const [expandedSources, setExpandedSources] = useState<Set<number>>(new Set());
  const [isLoading, setIsLoading] = useState(false);
  const [loadingSourceId, setLoadingSourceId] = useState<number | null>(null);
  const [showManualInput, setShowManualInput] = useState(false);
  const [manualText, setManualText] = useState("");
  const [manualName, setManualName] = useState("");
  const [viewingDoc, setViewingDoc] = useState<{ id: number; name: string; text: string } | null>(null);
  const [showChangeDetection, setShowChangeDetection] = useState(false);

  const { isOpen, options, confirm, handleConfirm, handleClose } = useConfirmDialog();

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const [sourcesData, docsData] = await Promise.all([
        api.listSources(),
        api.listDocuments({}),
      ]);
      setSources(sourcesData);
      setDocuments(docsData);
    } catch (e) {
      console.error("데이터 로드 실패:", e);
    }
  };

  const toggleSource = (sourceId: number) => {
    setExpandedSources((prev) => {
      const next = new Set(prev);
      if (next.has(sourceId)) {
        next.delete(sourceId);
      } else {
        next.add(sourceId);
      }
      return next;
    });
  };

  const getDocumentsForSource = (sourceId: number) => {
    return documents.filter((doc) => doc.source_id === sourceId);
  };

  const handleAddFolder = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "소스 폴더 선택",
    });

    if (selected) {
      setIsLoading(true);
      try {
        await api.addSource("folder", selected as string);
        await handleScan();
      } catch (e) {
        console.error("폴더 추가 실패:", e);
      } finally {
        setIsLoading(false);
      }
    }
  };

  const handleAddFile = async () => {
    const selected = await open({
      multiple: true,
      filters: [{ name: "텍스트 파일", extensions: ["txt", "md"] }],
      title: "파일 선택",
    });

    if (selected) {
      setIsLoading(true);
      try {
        const files = Array.isArray(selected) ? selected : [selected];
        for (const file of files) {
          await api.addSource("file", file);
        }
        await handleScan();
      } catch (e) {
        console.error("파일 추가 실패:", e);
      } finally {
        setIsLoading(false);
      }
    }
  };

  const handleAddManual = async () => {
    if (!manualText.trim()) return;
    setIsLoading(true);
    try {
      await api.addSource("manual", manualText, manualName || "수동 입력");
      setManualText("");
      setManualName("");
      setShowManualInput(false);
      await loadData();
    } catch (e) {
      console.error("수동 입력 실패:", e);
    } finally {
      setIsLoading(false);
    }
  };

  const handleScan = async () => {
    setIsLoading(true);
    try {
      const result = await api.scanSources();
      console.log("스캔 결과:", result);
      await loadData();
    } catch (e) {
      console.error("스캔 실패:", e);
    } finally {
      setIsLoading(false);
    }
  };

  const handleRescanSource = async (sourceId: number) => {
    setLoadingSourceId(sourceId);
    try {
      const result = await api.rescanSource(sourceId);
      console.log("소스 재스캔 결과:", result);
      await loadData();
    } catch (e) {
      console.error("소스 재스캔 실패:", e);
    } finally {
      setLoadingSourceId(null);
    }
  };

  const handleDeleteSource = async (source: Source) => {
    const confirmed = await confirm(confirmPresets.delete(source.display_name || source.path_or_key));
    if (!confirmed) return;

    try {
      await api.removeSource(source.id);
      await loadData();
    } catch (e) {
      console.error("소스 삭제 실패:", e);
    }
  };

  const handleDeleteDocument = async (doc: Document) => {
    const confirmed = await confirm(confirmPresets.delete(doc.display_name));
    if (!confirmed) return;

    try {
      await api.removeDocument(doc.id);
      await loadData();
    } catch (e) {
      console.error("문서 삭제 실패:", e);
    }
  };

  const handleViewManualText = async (doc: Document) => {
    try {
      const text = await api.getManualDocumentText(doc.id);
      setViewingDoc({ id: doc.id, name: doc.display_name, text });
    } catch (e) {
      console.error("텍스트 조회 실패:", e);
    }
  };

  const getSourceIcon = (sourceType: string) => {
    switch (sourceType) {
      case "folder":
        return <Folder className="h-4 w-4 text-blue-500" />;
      case "file":
        return <File className="h-4 w-4 text-green-500" />;
      case "manual":
        return <Edit3 className="h-4 w-4 text-purple-500" />;
      default:
        return <File className="h-4 w-4" />;
    }
  };

  const getSourceTypeLabel = (sourceType: string) => {
    switch (sourceType) {
      case "folder":
        return t("sources.folder");
      case "file":
        return t("sources.file");
      case "manual":
        return t("sources.manual");
      default:
        return sourceType;
    }
  };

  const statusColors: Record<string, string> = {
    unprocessed: "bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300",
    chunked: "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300",
    in_review: "bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300",
    approved: "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300",
    exported: "bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300",
    needs_attention: "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300",
  };

  const getStatusLabel = (status: string) => {
    const labels: Record<string, string> = {
      unprocessed: t("sources.status.unprocessed"),
      chunked: t("sources.status.chunked"),
      in_review: t("sources.status.inReview"),
      approved: t("sources.status.approved"),
      exported: t("sources.status.exported"),
      needs_attention: t("sources.status.needs_attention"),
    };
    return labels[status] || status;
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">{t("sources.title")}</h1>
        <div className="flex gap-2">
          <button
            onClick={handleAddFolder}
            disabled={isLoading}
            className="flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
          >
            <FolderPlus className="h-4 w-4" />
            {t("sources.addFolder")}
          </button>
          <button
            onClick={handleAddFile}
            disabled={isLoading}
            className="flex items-center gap-2 rounded-md bg-secondary px-3 py-2 text-sm hover:bg-secondary/80 disabled:opacity-50"
          >
            <FilePlus className="h-4 w-4" />
            {t("sources.addFile")}
          </button>
          <button
            onClick={() => setShowManualInput(true)}
            disabled={isLoading}
            className="flex items-center gap-2 rounded-md bg-secondary px-3 py-2 text-sm hover:bg-secondary/80 disabled:opacity-50"
          >
            <FileText className="h-4 w-4" />
            {t("sources.manualInput")}
          </button>
          <button
            onClick={handleScan}
            disabled={isLoading}
            className="flex items-center gap-2 rounded-md bg-secondary px-3 py-2 text-sm hover:bg-secondary/80 disabled:opacity-50"
          >
            <RefreshCw className={cn("h-4 w-4", isLoading && "animate-spin")} />
            {t("sources.scan")}
          </button>
          <button
            onClick={() => setShowChangeDetection(!showChangeDetection)}
            className={cn(
              "flex items-center gap-2 rounded-md px-3 py-2 text-sm",
              showChangeDetection
                ? "bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300"
                : "bg-secondary hover:bg-secondary/80"
            )}
          >
            <AlertTriangle className="h-4 w-4" />
            {t("sources.changeDetection")}
          </button>
        </div>
      </div>

      {/* 변경 감지 패널 */}
      {showChangeDetection && (
        <div className="rounded-lg border border-border bg-card p-4">
          <ChangeDetection onComplete={loadData} />
        </div>
      )}

      {/* 직접 입력 폼 */}
      {showManualInput && (
        <div className="rounded-lg border border-border bg-card p-4 space-y-4">
          <h3 className="font-medium">{t("sources.manualInput")}</h3>
          <input
            type="text"
            value={manualName}
            onChange={(e) => setManualName(e.target.value)}
            placeholder={t("sources.namePlaceholder")}
            className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
          />
          <textarea
            value={manualText}
            onChange={(e) => setManualText(e.target.value)}
            placeholder={t("sources.textPlaceholder")}
            rows={6}
            className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono"
          />
          <div className="flex justify-end gap-2">
            <button
              onClick={() => setShowManualInput(false)}
              className="rounded-md px-4 py-2 text-sm hover:bg-secondary"
            >
              {t("common.cancel")}
            </button>
            <button
              onClick={handleAddManual}
              disabled={!manualText.trim() || isLoading}
              className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              {t("sources.add")}
            </button>
          </div>
        </div>
      )}

      {/* 소스 및 문서 목록 */}
      <div className="rounded-lg border border-border overflow-hidden">
        {sources.length === 0 ? (
          <div className="px-4 py-8 text-center text-muted-foreground">
            {t("sources.noSources")} {t("sources.addSourceHint")}
          </div>
        ) : (
          <div className="divide-y divide-border">
            {sources.map((source) => {
              const sourceDocs = getDocumentsForSource(source.id);
              const isExpanded = expandedSources.has(source.id);
              const isSourceLoading = loadingSourceId === source.id;

              return (
                <div key={source.id}>
                  {/* 소스 헤더 */}
                  <div
                    className="flex items-center gap-3 px-4 py-3 bg-muted/30 hover:bg-muted/50 cursor-pointer"
                    onClick={() => toggleSource(source.id)}
                  >
                    <button className="p-0.5">
                      {isExpanded ? (
                        <ChevronDown className="h-4 w-4 text-muted-foreground" />
                      ) : (
                        <ChevronRight className="h-4 w-4 text-muted-foreground" />
                      )}
                    </button>
                    {getSourceIcon(source.type)}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="font-medium truncate">
                          {source.display_name || source.path_or_key}
                        </span>
                        <span className="text-xs text-muted-foreground px-1.5 py-0.5 bg-muted rounded">
                          {getSourceTypeLabel(source.type)}
                        </span>
                        <span className="text-xs text-muted-foreground">
                          {sourceDocs.length} {t("sources.documents")}
                        </span>
                      </div>
                      {source.type !== "manual" && (
                        <p className="text-xs text-muted-foreground truncate">
                          {source.path_or_key}
                        </p>
                      )}
                    </div>
                    <div className="flex items-center gap-1" onClick={(e) => e.stopPropagation()}>
                      {source.type !== "manual" && (
                        <button
                          onClick={() => handleRescanSource(source.id)}
                          disabled={isSourceLoading}
                          className="p-1.5 rounded hover:bg-secondary text-muted-foreground"
                          title={t("sources.rescan")}
                        >
                          <RefreshCw
                            className={cn("h-4 w-4", isSourceLoading && "animate-spin")}
                          />
                        </button>
                      )}
                      <button
                        onClick={() => handleDeleteSource(source)}
                        className="p-1.5 rounded hover:bg-secondary text-muted-foreground hover:text-destructive"
                        title={t("sources.remove")}
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    </div>
                  </div>

                  {/* 문서 목록 */}
                  {isExpanded && (
                    <div className="bg-background">
                      {sourceDocs.length === 0 ? (
                        <div className="px-12 py-4 text-sm text-muted-foreground">
                          {t("sources.noDocuments")}
                        </div>
                      ) : (
                        <table className="w-full">
                          <thead>
                            <tr className="border-b border-border text-xs text-muted-foreground">
                              <th className="px-12 py-2 text-left font-medium">{t("sources.name")}</th>
                              <th className="px-4 py-2 text-left font-medium">{t("sources.statusLabel")}</th>
                              <th className="px-4 py-2 text-left font-medium">{t("sources.size")}</th>
                              <th className="px-4 py-2 text-left font-medium">{t("sources.dateAdded")}</th>
                              <th className="px-4 py-2 text-right font-medium">{t("sources.actions")}</th>
                            </tr>
                          </thead>
                          <tbody>
                            {sourceDocs.map((doc) => (
                              <tr
                                key={doc.id}
                                className="border-b border-border last:border-0 hover:bg-muted/20"
                              >
                                <td className="px-12 py-2">
                                  <div>
                                    <p className="text-sm">{doc.display_name}</p>
                                    {doc.original_path && (
                                      <p className="truncate text-xs text-muted-foreground max-w-xs">
                                        {doc.original_path}
                                      </p>
                                    )}
                                  </div>
                                </td>
                                <td className="px-4 py-2">
                                  <span
                                    className={cn(
                                      "inline-flex rounded-full px-2 py-0.5 text-xs font-medium",
                                      statusColors[doc.status] || statusColors.unprocessed
                                    )}
                                  >
                                    {getStatusLabel(doc.status)}
                                  </span>
                                </td>
                                <td className="px-4 py-2 text-sm text-muted-foreground">
                                  {formatBytes(doc.byte_size)}
                                </td>
                                <td className="px-4 py-2 text-sm text-muted-foreground">
                                  {formatDate(doc.created_at)}
                                </td>
                                <td className="px-4 py-2 text-right">
                                  <div className="flex items-center justify-end gap-1">
                                    {source.type === "manual" && (
                                      <button
                                        onClick={() => handleViewManualText(doc)}
                                        className="p-1 rounded text-muted-foreground hover:bg-secondary hover:text-foreground"
                                        title={t("sources.viewText")}
                                      >
                                        <Eye className="h-4 w-4" />
                                      </button>
                                    )}
                                    <button
                                      onClick={() => handleDeleteDocument(doc)}
                                      className="p-1 rounded text-muted-foreground hover:bg-secondary hover:text-destructive"
                                      title={t("sources.removeDocument")}
                                    >
                                      <Trash2 className="h-4 w-4" />
                                    </button>
                                  </div>
                                </td>
                              </tr>
                            ))}
                          </tbody>
                        </table>
                      )}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* 직접입력 문서 텍스트 뷰어 */}
      {viewingDoc && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-background rounded-lg shadow-lg max-w-2xl w-full mx-4 max-h-[80vh] flex flex-col">
            <div className="flex items-center justify-between px-4 py-3 border-b border-border">
              <h3 className="font-medium">{viewingDoc.name}</h3>
              <button
                onClick={() => setViewingDoc(null)}
                className="p-1 rounded hover:bg-secondary"
              >
                <span className="sr-only">{t("common.close")}</span>
                &times;
              </button>
            </div>
            <div className="flex-1 overflow-auto p-4">
              <pre className="text-sm font-mono whitespace-pre-wrap break-words">
                {viewingDoc.text}
              </pre>
            </div>
          </div>
        </div>
      )}

      {/* 확인 다이얼로그 */}
      <ConfirmDialog
        isOpen={isOpen}
        onClose={handleClose}
        onConfirm={handleConfirm}
        title={options.title}
        message={options.message}
        confirmText={options.confirmText}
        cancelText={options.cancelText}
        variant={options.variant}
      />
    </div>
  );
}
