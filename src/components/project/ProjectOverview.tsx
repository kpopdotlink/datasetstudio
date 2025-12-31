import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { FileText, Layers, CheckCircle, Download } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useProjectStore } from "../../stores/projectStore";
import { formatNumber, formatDate } from "../../lib/utils";

export default function ProjectOverview() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { projectInfo, stats, refreshStats, isOpen } = useProjectStore();

  useEffect(() => {
    if (!isOpen) {
      navigate("/");
      return;
    }
    refreshStats();
  }, [isOpen, navigate, refreshStats]);

  if (!projectInfo) {
    return null;
  }

  const kpiCards = [
    {
      title: t("stats.documents"),
      value: formatNumber(stats?.document_count || 0),
      icon: FileText,
      color: "text-blue-500",
    },
    {
      title: t("stats.chunks"),
      value: formatNumber(stats?.chunk_count || 0),
      icon: Layers,
      color: "text-purple-500",
    },
    {
      title: t("stats.approvalRate"),
      value: `${Math.round(stats?.approval_rate || 0)}%`,
      icon: CheckCircle,
      color: "text-green-500",
    },
    {
      title: t("stats.exports"),
      value: formatNumber(stats?.export_count || 0),
      icon: Download,
      color: "text-orange-500",
    },
  ];

  return (
    <div className="space-y-6">
      {/* 프로젝트 정보 */}
      <div>
        <h1 className="text-2xl font-bold">{projectInfo.meta.name}</h1>
        <p className="text-sm text-muted-foreground">
          {t("overview.created")}: {formatDate(projectInfo.meta.created_at)} | {t("overview.version")}:{" "}
          {projectInfo.meta.version}
        </p>
      </div>

      {/* KPI 카드 */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {kpiCards.map((card) => (
          <div
            key={card.title}
            className="rounded-lg border border-border bg-card p-4"
          >
            <div className="flex items-center gap-3">
              <div className={`rounded-lg bg-secondary p-2 ${card.color}`}>
                <card.icon className="h-5 w-5" />
              </div>
              <div>
                <p className="text-sm text-muted-foreground">{card.title}</p>
                <p className="text-2xl font-bold">{card.value}</p>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* 빠른 액션 */}
      <div className="space-y-3">
        <h2 className="font-medium">{t("overview.quickStart")}</h2>
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
          <button
            onClick={() => navigate("/sources")}
            className="rounded-lg border border-border p-4 text-left hover:bg-secondary transition-colors"
          >
            <h3 className="font-medium">{t("overview.addSources")}</h3>
            <p className="text-sm text-muted-foreground">
              {t("overview.addSourcesDesc")}
            </p>
          </button>
          <button
            onClick={() => navigate("/chunks")}
            className="rounded-lg border border-border p-4 text-left hover:bg-secondary transition-colors"
          >
            <h3 className="font-medium">{t("overview.createChunks")}</h3>
            <p className="text-sm text-muted-foreground">
              {t("overview.createChunksDesc")}
            </p>
          </button>
          <button
            onClick={() => navigate("/review")}
            className="rounded-lg border border-border p-4 text-left hover:bg-secondary transition-colors"
          >
            <h3 className="font-medium">{t("overview.startReview")}</h3>
            <p className="text-sm text-muted-foreground">
              {t("overview.startReviewDesc")}
            </p>
          </button>
          <button
            onClick={() => navigate("/exports")}
            className="rounded-lg border border-border p-4 text-left hover:bg-secondary transition-colors"
          >
            <h3 className="font-medium">{t("overview.export")}</h3>
            <p className="text-sm text-muted-foreground">
              {t("overview.exportDesc")}
            </p>
          </button>
        </div>
      </div>

      {/* 통계 상세 */}
      {stats && (
        <div className="space-y-3">
          <h2 className="font-medium">{t("overview.detailedStats")}</h2>
          <div className="rounded-lg border border-border bg-card p-4">
            <div className="grid gap-4 sm:grid-cols-3">
              <div>
                <p className="text-sm text-muted-foreground">{t("stats.pendingReview")}</p>
                <p className="text-xl font-bold">
                  {formatNumber(stats.pending_count)}
                </p>
              </div>
              <div>
                <p className="text-sm text-muted-foreground">{t("stats.approvedChunks")}</p>
                <p className="text-xl font-bold text-green-500">
                  {formatNumber(stats.approved_count)}
                </p>
              </div>
              <div>
                <p className="text-sm text-muted-foreground">{t("overview.skippedRejected")}</p>
                <p className="text-xl font-bold text-yellow-500">
                  {formatNumber(stats.skipped_count + stats.rejected_count)}
                </p>
              </div>
              <div>
                <p className="text-sm text-muted-foreground">{t("stats.totalCharacters")}</p>
                <p className="text-xl font-bold">
                  {formatNumber(stats.total_chars)}
                </p>
              </div>
              <div>
                <p className="text-sm text-muted-foreground">{t("stats.totalTokens")}</p>
                <p className="text-xl font-bold">
                  {formatNumber(stats.total_tokens_est)}
                </p>
              </div>
              <div>
                <p className="text-sm text-muted-foreground">{t("stats.avgChunkLength")}</p>
                <p className="text-xl font-bold">
                  {Math.round(stats.avg_chunk_length)} {t("common.characters")}
                </p>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
