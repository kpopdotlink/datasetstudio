import { useState, useEffect, useCallback } from "react";
import { Plus, Trash2, Edit3, Upload, CheckCircle, XCircle, SkipForward, ChevronLeft, ChevronRight } from "lucide-react";
import { useTranslation } from "react-i18next";
import * as api from "../../api/dataset";
import { listPresets } from "../../api/export";
import { ManualEntry, ManualEntryListResult } from "../../api/dataset";
import { Preset } from "../../types";
import { open } from "@tauri-apps/plugin-dialog";
import { cn } from "../../lib/utils";

export default function DatasetManager() {
  const { t } = useTranslation();
  const [presets, setPresets] = useState<Preset[]>([]);
  const [selectedPresetId, setSelectedPresetId] = useState<number | null>(null);
  const [result, setResult] = useState<ManualEntryListResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  // 편집 다이얼로그
  const [showEditor, setShowEditor] = useState(false);
  const [editingEntry, setEditingEntry] = useState<ManualEntry | null>(null);
  const [editData, setEditData] = useState<Record<string, string>>({});
  const [presetFields, setPresetFields] = useState<string[]>([]);

  // 페이지네이션
  const [page, setPage] = useState(0);
  const pageSize = 20;

  useEffect(() => {
    loadPresets();
  }, []);

  useEffect(() => {
    if (selectedPresetId) {
      loadEntries();
      loadPresetFields();
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedPresetId, page]);

  const loadPresets = async () => {
    try {
      const p = await listPresets();
      setPresets(p);
    } catch (e) {
      console.error("프리셋 로드 실패:", e);
    }
  };

  const loadEntries = async () => {
    if (!selectedPresetId) return;
    setIsLoading(true);
    try {
      const r = await api.listManualEntries({
        preset_id: selectedPresetId,
        limit: pageSize,
        offset: page * pageSize,
      });
      setResult(r);
    } catch (e) {
      console.error("엔트리 로드 실패:", e);
    } finally {
      setIsLoading(false);
    }
  };

  const loadPresetFields = async () => {
    if (!selectedPresetId) return;
    try {
      const fields = await api.getPresetFields(selectedPresetId);
      setPresetFields(fields.length > 0 ? fields : getDefaultFields());
    } catch {
      setPresetFields(getDefaultFields());
    }
  };

  const getDefaultFields = (): string[] => {
    const preset = presets.find(p => p.id === selectedPresetId);
    if (!preset) return ["text"];

    try {
      const mapping = JSON.parse(preset.mapping_json);
      if (mapping.messages) return ["system_content", "user_content", "assistant_content"];
      if (mapping.conversations) return ["human_value", "gpt_value"];
      return Object.keys(mapping);
    } catch {
      return ["text"];
    }
  };

  const handleNewEntry = () => {
    setEditingEntry(null);
    const emptyData: Record<string, string> = {};
    presetFields.forEach(f => emptyData[f] = "");
    setEditData(emptyData);
    setShowEditor(true);
  };

  const handleEditEntry = (entry: ManualEntry) => {
    setEditingEntry(entry);
    try {
      const data = JSON.parse(entry.data_json);
      setEditData(data);
    } catch {
      const emptyData: Record<string, string> = {};
      presetFields.forEach(f => emptyData[f] = "");
      setEditData(emptyData);
    }
    setShowEditor(true);
  };

  const handleSaveEntry = async () => {
    if (!selectedPresetId) return;

    try {
      const dataJson = JSON.stringify(editData);
      if (editingEntry) {
        await api.updateManualEntry(editingEntry.id, dataJson);
      } else {
        await api.createManualEntry(selectedPresetId, dataJson);
      }
      setShowEditor(false);
      await loadEntries();
    } catch (e) {
      console.error("저장 실패:", e);
      alert("저장 실패: " + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleDeleteEntry = async (entry: ManualEntry) => {
    if (!confirm(t("manual.deleteConfirm"))) return;
    try {
      await api.deleteManualEntry(entry.id);
      await loadEntries();
    } catch (e) {
      console.error("삭제 실패:", e);
    }
  };

  const handleSetStatus = useCallback(async (entryId: number, status: string) => {
    try {
      await api.setManualEntryStatus(entryId, status);
      await loadEntries();
    } catch (e) {
      console.error("상태 변경 실패:", e);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedPresetId, page]);

  const handleImport = async () => {
    if (!selectedPresetId) return;

    try {
      const file = await open({
        filters: [{ name: "JSONL", extensions: ["jsonl", "json"] }],
      });

      if (!file) return;

      const count = await api.importJsonl(selectedPresetId, file as string);
      alert(t("manual.importSuccess", { count }));
      await loadEntries();
    } catch (e) {
      console.error("Import 실패:", e);
      alert("Import 실패: " + (e instanceof Error ? e.message : String(e)));
    }
  };

  const totalPages = result ? Math.ceil(result.total / pageSize) : 0;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">{t("manual.title")}</h1>
        <div className="flex items-center gap-2">
          <button
            onClick={handleImport}
            disabled={!selectedPresetId}
            className="flex items-center gap-2 rounded-md bg-secondary px-3 py-2 text-sm hover:bg-secondary/80 disabled:opacity-50"
          >
            <Upload className="h-4 w-4" />
            {t("manual.importJsonl")}
          </button>
          <button
            onClick={handleNewEntry}
            disabled={!selectedPresetId}
            className="flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
          >
            <Plus className="h-4 w-4" />
            {t("manual.newEntry")}
          </button>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-4">
        {/* 프리셋 선택 */}
        <div className="lg:col-span-1 space-y-4">
          <div className="rounded-lg border border-border bg-card p-4">
            <h2 className="font-medium mb-3">{t("manual.selectPreset")}</h2>
            <div className="space-y-2">
              {presets.map((preset) => (
                <button
                  key={preset.id}
                  onClick={() => {
                    setSelectedPresetId(preset.id);
                    setPage(0);
                  }}
                  className={cn(
                    "w-full rounded-md border p-2 text-left text-sm transition-colors",
                    selectedPresetId === preset.id
                      ? "border-primary bg-primary/5"
                      : "border-border hover:bg-secondary"
                  )}
                >
                  <p className="font-medium">{preset.name}</p>
                  <p className="text-xs text-muted-foreground">
                    {preset.is_builtin ? t("manual.builtin") : t("manual.custom")}
                  </p>
                </button>
              ))}
            </div>
          </div>

          {/* 통계 */}
          {result && (
            <div className="rounded-lg border border-border bg-card p-4">
              <h2 className="font-medium mb-3">{t("stats.title")}</h2>
              <div className="grid grid-cols-2 gap-2 text-sm">
                <div>
                  <p className="text-muted-foreground">{t("manual.stats.total")}</p>
                  <p className="font-medium">{result.total}</p>
                </div>
                <div>
                  <p className="text-muted-foreground">{t("manual.stats.pending")}</p>
                  <p className="font-medium">{result.pending}</p>
                </div>
                <div>
                  <p className="text-muted-foreground">{t("manual.stats.approved")}</p>
                  <p className="font-medium text-green-600">{result.approved}</p>
                </div>
                <div>
                  <p className="text-muted-foreground">{t("manual.stats.rejected")}</p>
                  <p className="font-medium text-red-600">{result.rejected}</p>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* 엔트리 목록 */}
        <div className="lg:col-span-3">
          {!selectedPresetId ? (
            <div className="flex h-64 items-center justify-center rounded-lg border border-dashed border-border">
              <p className="text-muted-foreground">{t("manual.noPreset")}</p>
            </div>
          ) : isLoading ? (
            <div className="flex h-64 items-center justify-center">
              <p className="text-muted-foreground">{t("common.loading")}</p>
            </div>
          ) : result && result.entries.length === 0 ? (
            <div className="flex h-64 flex-col items-center justify-center gap-4 rounded-lg border border-dashed border-border">
              <p className="text-muted-foreground">{t("manual.noEntries")}</p>
              <button
                onClick={handleNewEntry}
                className="flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm text-primary-foreground"
              >
                <Plus className="h-4 w-4" />
                {t("manual.addNewEntry")}
              </button>
            </div>
          ) : (
            <div className="space-y-4">
              <div className="space-y-2">
                {result?.entries.map((entry) => {
                  let data: Record<string, string> = {};
                  try {
                    data = JSON.parse(entry.data_json);
                  } catch {
                    // ignore
                  }
                  return (
                    <div
                      key={entry.id}
                      className="rounded-lg border border-border bg-card p-4"
                    >
                      <div className="flex items-start justify-between gap-4">
                        <div className="flex-1 min-w-0">
                          {Object.entries(data).slice(0, 3).map(([key, value]) => (
                            <div key={key} className="mb-2">
                              <p className="text-xs font-medium text-muted-foreground">{key}</p>
                              <p className="text-sm line-clamp-2">{String(value)}</p>
                            </div>
                          ))}
                          {Object.keys(data).length > 3 && (
                            <p className="text-xs text-muted-foreground">
                              {t("manual.moreFields", { count: Object.keys(data).length - 3 })}
                            </p>
                          )}
                        </div>
                        <div className="flex items-center gap-1">
                          {/* 상태 버튼 */}
                          <button
                            onClick={() => handleSetStatus(entry.id, "approved")}
                            className={cn(
                              "p-1.5 rounded",
                              entry.review_status === "approved"
                                ? "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300"
                                : "hover:bg-secondary"
                            )}
                            title={t("review.approve")}
                          >
                            <CheckCircle className="h-4 w-4" />
                          </button>
                          <button
                            onClick={() => handleSetStatus(entry.id, "skipped")}
                            className={cn(
                              "p-1.5 rounded",
                              entry.review_status === "skipped"
                                ? "bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300"
                                : "hover:bg-secondary"
                            )}
                            title={t("review.skip")}
                          >
                            <SkipForward className="h-4 w-4" />
                          </button>
                          <button
                            onClick={() => handleSetStatus(entry.id, "rejected")}
                            className={cn(
                              "p-1.5 rounded",
                              entry.review_status === "rejected"
                                ? "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300"
                                : "hover:bg-secondary"
                            )}
                            title={t("review.reject")}
                          >
                            <XCircle className="h-4 w-4" />
                          </button>
                          <div className="w-px h-4 bg-border mx-1" />
                          <button
                            onClick={() => handleEditEntry(entry)}
                            className="p-1.5 rounded hover:bg-secondary"
                            title={t("common.edit")}
                          >
                            <Edit3 className="h-4 w-4" />
                          </button>
                          <button
                            onClick={() => handleDeleteEntry(entry)}
                            className="p-1.5 rounded hover:bg-destructive/10 text-destructive"
                            title={t("common.delete")}
                          >
                            <Trash2 className="h-4 w-4" />
                          </button>
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>

              {/* 페이지네이션 */}
              {totalPages > 1 && (
                <div className="flex items-center justify-center gap-2">
                  <button
                    onClick={() => setPage(p => Math.max(0, p - 1))}
                    disabled={page === 0}
                    className="rounded p-2 hover:bg-secondary disabled:opacity-50"
                  >
                    <ChevronLeft className="h-4 w-4" />
                  </button>
                  <span className="text-sm">
                    {page + 1} / {totalPages}
                  </span>
                  <button
                    onClick={() => setPage(p => Math.min(totalPages - 1, p + 1))}
                    disabled={page >= totalPages - 1}
                    className="rounded p-2 hover:bg-secondary disabled:opacity-50"
                  >
                    <ChevronRight className="h-4 w-4" />
                  </button>
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {/* 편집 다이얼로그 */}
      {showEditor && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-background rounded-lg shadow-lg max-w-2xl w-full mx-4 p-6 max-h-[80vh] flex flex-col">
            <h3 className="font-medium text-lg mb-4">
              {editingEntry ? t("manual.editEntry") : t("manual.newEntryTitle")}
            </h3>

            <div className="flex-1 overflow-y-auto space-y-4">
              {presetFields.map((field) => (
                <div key={field}>
                  <label className="text-sm font-medium block mb-1">
                    {field}
                  </label>
                  <textarea
                    value={editData[field] || ""}
                    onChange={(e) => setEditData({ ...editData, [field]: e.target.value })}
                    className="w-full h-24 rounded-md border border-border bg-background px-3 py-2 text-sm resize-none"
                    placeholder={t("manual.inputPlaceholder", { field })}
                  />
                </div>
              ))}
            </div>

            <div className="flex justify-end gap-2 mt-4 pt-4 border-t border-border">
              <button
                onClick={() => setShowEditor(false)}
                className="px-4 py-2 text-sm rounded hover:bg-secondary"
              >
                {t("common.cancel")}
              </button>
              <button
                onClick={handleSaveEntry}
                className="px-4 py-2 text-sm bg-primary text-primary-foreground rounded hover:bg-primary/90"
              >
                {t("common.save")}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
