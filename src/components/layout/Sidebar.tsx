import { NavLink, useNavigate } from "react-router-dom";
import {
  Home,
  FolderOpen,
  Layers,
  CheckSquare,
  Download,
  Database,
  X,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { useProjectStore } from "../../stores/projectStore";
import { cn } from "../../lib/utils";

export default function Sidebar() {
  const { t } = useTranslation();
  const { projectInfo, closeProject } = useProjectStore();
  const navigate = useNavigate();

  const navItems = [
    { to: "/project", icon: Home, label: t("stats.overview") },
    { to: "/sources", icon: FolderOpen, label: t("nav.sources") },
    { to: "/chunks", icon: Layers, label: t("nav.chunks") },
    { to: "/review", icon: CheckSquare, label: t("nav.review") },
    { to: "/dataset", icon: Database, label: t("nav.manual") },
    { to: "/exports", icon: Download, label: t("nav.export") },
  ];

  const handleClose = async () => {
    await closeProject();
    navigate("/");
  };

  return (
    <aside className="flex w-56 flex-col border-r border-border bg-card">
      {/* 프로젝트 헤더 */}
      <div className="flex items-center justify-between border-b border-border p-4">
        <div className="min-w-0 flex-1">
          <h1 className="truncate text-sm font-semibold">
            {projectInfo?.meta.name || t("project.title")}
          </h1>
          <p className="truncate text-xs text-muted-foreground">
            {projectInfo?.path || t("common.noData")}
          </p>
        </div>
        {projectInfo && (
          <button
            onClick={handleClose}
            className="ml-2 rounded p-1 hover:bg-secondary"
            title={t("project.close")}
          >
            <X className="h-4 w-4" />
          </button>
        )}
      </div>

      {/* 네비게이션 */}
      <nav className="flex-1 space-y-1 p-2">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) =>
              cn(
                "flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors",
                isActive
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:bg-secondary hover:text-foreground"
              )
            }
          >
            <item.icon className="h-4 w-4" />
            {item.label}
          </NavLink>
        ))}
      </nav>

      {/* 통계 요약 */}
      {projectInfo && (
        <div className="border-t border-border p-4">
          <div className="grid grid-cols-2 gap-2 text-xs">
            <div>
              <p className="text-muted-foreground">{t("stats.documents")}</p>
              <p className="font-medium">{projectInfo.document_count}</p>
            </div>
            <div>
              <p className="text-muted-foreground">{t("stats.chunks")}</p>
              <p className="font-medium">{projectInfo.chunk_count}</p>
            </div>
            <div className="col-span-2">
              <p className="text-muted-foreground">{t("stats.approvalRate")}</p>
              <p className="font-medium">
                {projectInfo.chunk_count > 0
                  ? Math.round(
                      (projectInfo.approved_count / projectInfo.chunk_count) *
                        100
                    )
                  : 0}
                %
              </p>
            </div>
          </div>
        </div>
      )}
    </aside>
  );
}
