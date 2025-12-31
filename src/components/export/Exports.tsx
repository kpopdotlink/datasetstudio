import { useState, useEffect } from "react";
import { Download, FolderOpen, Check, Upload, Copy, CheckCheck, Plus, Pencil, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import * as api from "../../api/export";
import * as sourceApi from "../../api/source";
import { UploadCommand } from "../../api/export";
import { Preset, ExportRun, ExportFilters, Source } from "../../types";
import { useJobStore } from "../../stores/jobStore";
import { formatDate, cn } from "../../lib/utils";

export default function Exports() {
  const { t } = useTranslation();
  const { addJob } = useJobStore();
  const [presets, setPresets] = useState<Preset[]>([]);
  const [exportRuns, setExportRuns] = useState<ExportRun[]>([]);
  const [sources, setSources] = useState<Source[]>([]);
  const [selectedSourceIds, setSelectedSourceIds] = useState<Set<number>>(new Set());
  const [selectedPreset, setSelectedPreset] = useState<number | null>(null);
  const [filters, setFilters] = useState<ExportFilters>({
    approved_only: true,
    include_manual_entries: true,
    split_by_source: false,
  });
  const [isLoading, setIsLoading] = useState(false);

  // Upload commands dialog state
  const [showUploadDialog, setShowUploadDialog] = useState(false);
  const [uploadCommands, setUploadCommands] = useState<UploadCommand[]>([]);
  const [selectedExportId, setSelectedExportId] = useState<string>("");
  const [datasetName, setDatasetName] = useState<string>("");
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);

  // Preset editor dialog state
  const [showPresetDialog, setShowPresetDialog] = useState(false);
  const [editingPreset, setEditingPreset] = useState<Preset | null>(null);
  const [presetName, setPresetName] = useState("");
  const [presetMapping, setPresetMapping] = useState("");
  const [presetValidators, setPresetValidators] = useState("");
  const [presetError, setPresetError] = useState<string | null>(null);

  useEffect(() => {
    loadData();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const loadData = async () => {
    try {
      const [p, r, s] = await Promise.all([
        api.listPresets(),
        api.listExportRuns(),
        sourceApi.listSources(),
      ]);
      setPresets(p);
      setExportRuns(r);
      setSources(s);
      if (p.length > 0 && selectedPreset === null) {
        setSelectedPreset(p[0].id);
      }
    } catch (e) {
      console.error("데이터 로드 실패:", e);
    }
  };

  const handleExport = async () => {
    if (!selectedPreset) return;
    setIsLoading(true);
    try {
      // 선택된 소스 필터 적용
      const exportFilters: ExportFilters = {
        ...filters,
        source_ids: selectedSourceIds.size > 0 ? Array.from(selectedSourceIds) : undefined,
      };
      const jobId = await api.startExportJob(selectedPreset, exportFilters);
      addJob(jobId, "export");
      // 잡 완료 후 목록 새로고침
      setTimeout(loadData, 1000);
    } catch (e) {
      console.error("Export 실패:", e);
    } finally {
      setIsLoading(false);
    }
  };

  const toggleSourceSelection = (sourceId: number) => {
    const newSet = new Set(selectedSourceIds);
    if (newSet.has(sourceId)) {
      newSet.delete(sourceId);
    } else {
      newSet.add(sourceId);
    }
    setSelectedSourceIds(newSet);
  };

  const handleOpenFolder = async () => {
    try {
      await api.openExportFolder();
    } catch (e) {
      console.error("폴더 열기 실패:", e);
    }
  };

  const handleShowUploadCommands = (exportId: string) => {
    setSelectedExportId(exportId);
    setDatasetName("username/my-dataset");
    setUploadCommands([]);
    setShowUploadDialog(true);
  };

  const handleGenerateCommands = async () => {
    if (!selectedExportId || !datasetName.trim()) return;
    try {
      const commands = await api.generateUploadCommands(selectedExportId, datasetName.trim());
      setUploadCommands(commands);
    } catch (e) {
      console.error("명령 생성 실패:", e);
    }
  };

  const handleCopyCommand = async (command: string, index: number) => {
    try {
      await navigator.clipboard.writeText(command);
      setCopiedIndex(index);
      setTimeout(() => setCopiedIndex(null), 2000);
    } catch (e) {
      console.error("복사 실패:", e);
    }
  };

  // 새 프리셋 생성 다이얼로그 열기
  const handleNewPreset = () => {
    setEditingPreset(null);
    setPresetName("");
    setPresetMapping(JSON.stringify({ text: "content" }, null, 2));
    setPresetValidators("");
    setPresetError(null);
    setShowPresetDialog(true);
  };

  // 프리셋 편집 다이얼로그 열기
  const handleEditPreset = (preset: Preset) => {
    setEditingPreset(preset);
    setPresetName(preset.name);
    try {
      setPresetMapping(JSON.stringify(JSON.parse(preset.mapping_json), null, 2));
    } catch {
      setPresetMapping(preset.mapping_json);
    }
    setPresetValidators(preset.validators_json || "");
    setPresetError(null);
    setShowPresetDialog(true);
  };

  // 프리셋 저장
  const handleSavePreset = async () => {
    setPresetError(null);

    if (!presetName.trim()) {
      setPresetError(t("export.presetNameRequired"));
      return;
    }

    // JSON 유효성 검사
    try {
      JSON.parse(presetMapping);
    } catch {
      setPresetError(t("export.invalidMappingJson"));
      return;
    }

    if (presetValidators.trim()) {
      try {
        JSON.parse(presetValidators);
      } catch {
        setPresetError(t("export.invalidValidatorsJson"));
        return;
      }
    }

    try {
      if (editingPreset) {
        await api.updatePreset(
          editingPreset.id,
          presetName.trim(),
          presetMapping,
          presetValidators.trim() || undefined
        );
      } else {
        await api.createPreset(
          presetName.trim(),
          presetMapping,
          presetValidators.trim() || undefined
        );
      }
      setShowPresetDialog(false);
      await loadData();
    } catch (e) {
      console.error("Preset save failed:", e);
      setPresetError(t("export.presetSaveFailed", { error: e instanceof Error ? e.message : String(e) }));
    }
  };

  // 프리셋 삭제
  const handleDeletePreset = async (preset: Preset) => {
    if (!confirm(t("export.deletePresetConfirm", { name: preset.name }))) return;

    try {
      await api.deletePreset(preset.id);
      if (selectedPreset === preset.id) {
        setSelectedPreset(null);
      }
      await loadData();
    } catch (e) {
      console.error("프리셋 삭제 실패:", e);
      alert("프리셋 삭제 실패: " + (e instanceof Error ? e.message : String(e)));
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">{t("export.title")}</h1>
        <button
          onClick={handleOpenFolder}
          className="flex items-center gap-2 rounded-md bg-secondary px-3 py-2 text-sm hover:bg-secondary/80"
        >
          <FolderOpen className="h-4 w-4" />
          {t("export.openFolder")}
        </button>
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        {/* Export 설정 */}
        <div className="space-y-6 rounded-lg border border-border bg-card p-6">
          <h2 className="font-medium">{t("export.settings")}</h2>

          {/* 프리셋 선택 */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <label className="text-sm text-muted-foreground">{t("export.preset")}</label>
              <button
                onClick={handleNewPreset}
                className="flex items-center gap-1 rounded px-2 py-1 text-xs hover:bg-secondary"
              >
                <Plus className="h-3 w-3" />
                {t("export.newPreset")}
              </button>
            </div>
            <div className="space-y-2">
              {presets.map((preset) => (
                <div
                  key={preset.id}
                  className={cn(
                    "rounded-md border p-3 transition-colors",
                    selectedPreset === preset.id
                      ? "border-primary bg-primary/5"
                      : "border-border hover:bg-secondary"
                  )}
                >
                  <div className="flex items-center justify-between">
                    <button
                      onClick={() => setSelectedPreset(preset.id)}
                      className="flex-1 text-left"
                    >
                      <p className="font-medium">{preset.name}</p>
                      <p className="text-xs text-muted-foreground">
                        {preset.is_builtin ? t("export.builtinPreset") : t("export.customPreset")}
                      </p>
                    </button>
                    <div className="flex items-center gap-1">
                      {!preset.is_builtin && (
                        <>
                          <button
                            onClick={() => handleEditPreset(preset)}
                            className="p-1 rounded hover:bg-secondary"
                            title={t("common.edit")}
                          >
                            <Pencil className="h-3.5 w-3.5" />
                          </button>
                          <button
                            onClick={() => handleDeletePreset(preset)}
                            className="p-1 rounded hover:bg-destructive/10 text-destructive"
                            title={t("common.delete")}
                          >
                            <Trash2 className="h-3.5 w-3.5" />
                          </button>
                        </>
                      )}
                      {selectedPreset === preset.id && (
                        <Check className="h-4 w-4 text-primary ml-1" />
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* 소스(폴더) 선택 */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <label className="text-sm text-muted-foreground">{t("export.sourceSelection")}</label>
              <div className="flex gap-2">
                <button
                  onClick={() => setSelectedSourceIds(new Set(sources.map(s => s.id)))}
                  className="text-xs text-primary hover:underline"
                >
                  {t("common.selectAll")}
                </button>
                <button
                  onClick={() => setSelectedSourceIds(new Set())}
                  className="text-xs text-muted-foreground hover:underline"
                >
                  {t("common.deselectAll")}
                </button>
              </div>
            </div>
            <div className="max-h-[150px] overflow-y-auto border border-border rounded-md">
              {sources.length === 0 ? (
                <p className="p-3 text-sm text-muted-foreground">{t("sources.noSources")}</p>
              ) : (
                sources.map((source) => (
                  <label
                    key={source.id}
                    className={cn(
                      "flex items-center gap-2 px-3 py-2 cursor-pointer hover:bg-secondary border-b border-border last:border-0",
                      selectedSourceIds.has(source.id) && "bg-primary/5"
                    )}
                  >
                    <input
                      type="checkbox"
                      checked={selectedSourceIds.has(source.id)}
                      onChange={() => toggleSourceSelection(source.id)}
                      className="h-4 w-4 rounded"
                    />
                    <span className="text-sm truncate">
                      {source.display_name || source.path_or_key}
                    </span>
                  </label>
                ))
              )}
            </div>
            <p className="text-xs text-muted-foreground">
              {selectedSourceIds.size === 0
                ? t("export.allSourcesExport")
                : t("export.sourcesSelected", { count: selectedSourceIds.size })}
            </p>
          </div>

          {/* 필터 */}
          <div className="space-y-3">
            <label className="text-sm text-muted-foreground">{t("export.options")}</label>
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={filters.approved_only}
                onChange={(e) =>
                  setFilters({ ...filters, approved_only: e.target.checked })
                }
                className="h-4 w-4 rounded"
              />
              <span className="text-sm">{t("export.approvedOnly")}</span>
            </label>
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={filters.include_manual_entries ?? true}
                onChange={(e) =>
                  setFilters({ ...filters, include_manual_entries: e.target.checked })
                }
                className="h-4 w-4 rounded"
              />
              <span className="text-sm">{t("export.includeManual")}</span>
            </label>
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={filters.split_by_source ?? false}
                onChange={(e) =>
                  setFilters({ ...filters, split_by_source: e.target.checked })
                }
                className="h-4 w-4 rounded"
              />
              <span className="text-sm">{t("export.splitBySource")}</span>
            </label>
          </div>

          <button
            onClick={handleExport}
            disabled={!selectedPreset || isLoading}
            className="flex w-full items-center justify-center gap-2 rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
          >
            <Download className="h-4 w-4" />
            {t("export.exportJsonl")}
          </button>
        </div>

        {/* Export 이력 */}
        <div className="space-y-4 rounded-lg border border-border bg-card p-6">
          <h2 className="font-medium">{t("export.history")}</h2>

          {exportRuns.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              {t("export.noHistory")}
            </p>
          ) : (
            <div className="space-y-3 max-h-[400px] overflow-y-auto">
              {exportRuns.map((run) => {
                const stats = run.stats_json ? JSON.parse(run.stats_json) : null;
                // Export ID는 created_at 기반으로 생성됨 (예: 2025-01-01T12-30-45Z)
                const exportId = run.created_at.replace(/:/g, "-").replace(" ", "T") + "Z";
                return (
                  <div
                    key={run.id}
                    className="rounded-md border border-border p-3"
                  >
                    <div className="flex items-center justify-between">
                      <span className="font-medium">{run.preset_name}</span>
                      <div className="flex items-center gap-2">
                        <button
                          onClick={() => handleShowUploadCommands(exportId)}
                          className="p-1 rounded hover:bg-secondary"
                          title="업로드 명령 보기"
                        >
                          <Upload className="h-4 w-4" />
                        </button>
                        <span className="text-xs text-muted-foreground">
                          {formatDate(run.created_at)}
                        </span>
                      </div>
                    </div>
                    {stats && (
                      <div className="mt-2 flex gap-4 text-xs text-muted-foreground">
                        <span>{stats.total_lines} {t("common.lines")}</span>
                        <span>{stats.total_chars} {t("common.characters")}</span>
                        <span>~{stats.token_est_total} {t("common.tokens")}</span>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>

      {/* Upload Commands Dialog */}
      {showUploadDialog && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-background rounded-lg shadow-lg max-w-2xl w-full mx-4 p-6 max-h-[80vh] flex flex-col">
            <h3 className="font-medium text-lg mb-4">{t("export.uploadCommands")}</h3>

            {/* Dataset name input */}
            <div className="mb-4">
              <label className="text-sm text-muted-foreground block mb-2">
                {t("export.datasetName")}
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={datasetName}
                  onChange={(e) => setDatasetName(e.target.value)}
                  className="flex-1 rounded-md border border-border bg-background px-3 py-2 text-sm"
                  placeholder="username/my-dataset"
                />
                <button
                  onClick={handleGenerateCommands}
                  className="px-4 py-2 text-sm bg-primary text-primary-foreground rounded hover:bg-primary/90"
                >
                  {t("export.generate")}
                </button>
              </div>
            </div>

            {/* Commands list */}
            <div className="flex-1 overflow-y-auto space-y-3">
              {uploadCommands.length === 0 ? (
                <p className="text-sm text-muted-foreground text-center py-8">
                  {t("export.generateHint")}
                </p>
              ) : (
                uploadCommands.map((cmd, index) => (
                  <div key={cmd.platform} className="rounded-md border border-border p-4">
                    <div className="flex items-center justify-between mb-2">
                      <span className="font-medium">{cmd.name}</span>
                      <button
                        onClick={() => handleCopyCommand(cmd.command, index)}
                        className="p-1.5 rounded hover:bg-secondary"
                        title="복사"
                      >
                        {copiedIndex === index ? (
                          <CheckCheck className="h-4 w-4 text-green-500" />
                        ) : (
                          <Copy className="h-4 w-4" />
                        )}
                      </button>
                    </div>
                    <p className="text-xs text-muted-foreground mb-2">{cmd.description}</p>
                    <pre className="text-xs bg-muted rounded p-3 overflow-x-auto whitespace-pre-wrap">
                      {cmd.command}
                    </pre>
                  </div>
                ))
              )}
            </div>

            {/* Close button */}
            <div className="flex justify-end mt-4 pt-4 border-t border-border">
              <button
                onClick={() => setShowUploadDialog(false)}
                className="px-4 py-2 text-sm rounded hover:bg-secondary"
              >
                {t("common.close")}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Preset Editor Dialog */}
      {showPresetDialog && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-background rounded-lg shadow-lg max-w-2xl w-full mx-4 p-6 max-h-[80vh] flex flex-col">
            <h3 className="font-medium text-lg mb-4">
              {editingPreset ? t("export.editPreset") : t("export.newPresetTitle")}
            </h3>

            <div className="flex-1 overflow-y-auto space-y-4">
              {/* 프리셋 이름 */}
              <div>
                <label className="text-sm text-muted-foreground block mb-2">
                  {t("export.presetName")}
                </label>
                <input
                  type="text"
                  value={presetName}
                  onChange={(e) => setPresetName(e.target.value)}
                  className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm"
                  placeholder="my-custom-preset"
                />
              </div>

              {/* 매핑 JSON */}
              <div>
                <label className="text-sm text-muted-foreground block mb-2">
                  {t("export.mappingJson")}
                </label>
                <p className="text-xs text-muted-foreground mb-2">
                  {t("export.mappingHint")}
                </p>
                <textarea
                  value={presetMapping}
                  onChange={(e) => setPresetMapping(e.target.value)}
                  className="w-full h-40 rounded-md border border-border bg-background px-3 py-2 text-sm font-mono resize-none"
                  placeholder='{"text": "content"}'
                />
              </div>

              {/* 검증기 JSON (선택) */}
              <div>
                <label className="text-sm text-muted-foreground block mb-2">
                  {t("export.validatorsJson")}
                </label>
                <textarea
                  value={presetValidators}
                  onChange={(e) => setPresetValidators(e.target.value)}
                  className="w-full h-24 rounded-md border border-border bg-background px-3 py-2 text-sm font-mono resize-none"
                  placeholder='{"min_length": 10}'
                />
              </div>

              {/* 에러 메시지 */}
              {presetError && (
                <p className="text-sm text-destructive">{presetError}</p>
              )}

              {/* 예시 프리셋 */}
              <div className="bg-muted rounded-md p-3">
                <p className="text-xs font-medium mb-2">{t("export.examples")}:</p>
                <pre className="text-xs overflow-x-auto whitespace-pre-wrap">{`// Text-only
{"text": "content"}

// Prompt/Completion
{"prompt": "prefix", "completion": "content"}

// Chat (messages 배열)
{
  "messages": [
    {"role": "user", "content": "content"},
    {"role": "assistant", "content": "response"}
  ]
}`}</pre>
              </div>
            </div>

            {/* 버튼 */}
            <div className="flex justify-end gap-2 mt-4 pt-4 border-t border-border">
              <button
                onClick={() => setShowPresetDialog(false)}
                className="px-4 py-2 text-sm rounded hover:bg-secondary"
              >
                {t("common.cancel")}
              </button>
              <button
                onClick={handleSavePreset}
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
