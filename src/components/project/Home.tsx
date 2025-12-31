import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { FolderPlus, FolderOpen, Clock } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { useProjectStore } from "../../stores/projectStore";
import { cn } from "../../lib/utils";

export default function Home() {
  const navigate = useNavigate();
  const { createProject, openProject, recentProjects, isLoading } =
    useProjectStore();
  const [showCreate, setShowCreate] = useState(false);
  const [newProjectName, setNewProjectName] = useState("");
  const [newProjectPath, setNewProjectPath] = useState("");

  const handleCreate = async () => {
    if (!newProjectName || !newProjectPath) return;
    try {
      await createProject(newProjectPath, newProjectName);
      navigate("/project");
    } catch (e) {
      console.error("프로젝트 생성 실패:", e);
    }
  };

  const handleOpen = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "프로젝트 폴더 선택",
    });

    if (selected) {
      try {
        await openProject(selected as string);
        navigate("/project");
      } catch (e) {
        console.error("프로젝트 열기 실패:", e);
      }
    }
  };

  const handleOpenRecent = async (path: string) => {
    try {
      await openProject(path);
      navigate("/project");
    } catch (e) {
      console.error("프로젝트 열기 실패:", e);
    }
  };

  const handleSelectPath = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "프로젝트 위치 선택",
    });

    if (selected) {
      setNewProjectPath(selected as string);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-8">
      <div className="w-full max-w-2xl space-y-8">
        {/* 로고 */}
        <div className="text-center">
          <h1 className="text-3xl font-bold">Dataset Studio</h1>
          <p className="mt-2 text-muted-foreground">
            LLM 학습용 텍스트 데이터셋 제작 도구
          </p>
        </div>

        {/* 액션 버튼 */}
        <div className="grid gap-4 sm:grid-cols-2">
          <button
            onClick={() => setShowCreate(true)}
            disabled={isLoading}
            className={cn(
              "flex items-center gap-3 rounded-lg border border-border p-6 text-left transition-colors hover:bg-secondary",
              isLoading && "opacity-50 cursor-not-allowed"
            )}
          >
            <div className="rounded-lg bg-primary/10 p-3">
              <FolderPlus className="h-6 w-6 text-primary" />
            </div>
            <div>
              <h2 className="font-medium">새 프로젝트</h2>
              <p className="text-sm text-muted-foreground">
                새로운 데이터셋 프로젝트 생성
              </p>
            </div>
          </button>

          <button
            onClick={handleOpen}
            disabled={isLoading}
            className={cn(
              "flex items-center gap-3 rounded-lg border border-border p-6 text-left transition-colors hover:bg-secondary",
              isLoading && "opacity-50 cursor-not-allowed"
            )}
          >
            <div className="rounded-lg bg-primary/10 p-3">
              <FolderOpen className="h-6 w-6 text-primary" />
            </div>
            <div>
              <h2 className="font-medium">프로젝트 열기</h2>
              <p className="text-sm text-muted-foreground">
                기존 프로젝트 폴더 선택
              </p>
            </div>
          </button>
        </div>

        {/* 새 프로젝트 폼 */}
        {showCreate && (
          <div className="rounded-lg border border-border bg-card p-6 space-y-4">
            <h3 className="font-medium">새 프로젝트 생성</h3>
            <div className="space-y-2">
              <label className="text-sm text-muted-foreground">
                프로젝트 이름
              </label>
              <input
                type="text"
                value={newProjectName}
                onChange={(e) => setNewProjectName(e.target.value)}
                placeholder="My Dataset"
                className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm text-muted-foreground">
                프로젝트 위치
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={newProjectPath}
                  readOnly
                  placeholder="폴더를 선택하세요"
                  className="flex-1 rounded-md border border-input bg-background px-3 py-2 text-sm"
                />
                <button
                  onClick={handleSelectPath}
                  className="rounded-md bg-secondary px-3 py-2 text-sm hover:bg-secondary/80"
                >
                  선택
                </button>
              </div>
            </div>
            <div className="flex justify-end gap-2">
              <button
                onClick={() => setShowCreate(false)}
                className="rounded-md px-4 py-2 text-sm hover:bg-secondary"
              >
                취소
              </button>
              <button
                onClick={handleCreate}
                disabled={!newProjectName || !newProjectPath || isLoading}
                className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
              >
                생성
              </button>
            </div>
          </div>
        )}

        {/* 최근 프로젝트 */}
        {recentProjects.length > 0 && (
          <div className="space-y-3">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Clock className="h-4 w-4" />
              <span>최근 프로젝트</span>
            </div>
            <div className="space-y-2">
              {recentProjects.map((path) => (
                <button
                  key={path}
                  onClick={() => handleOpenRecent(path)}
                  disabled={isLoading}
                  className="w-full rounded-md border border-border p-3 text-left text-sm hover:bg-secondary transition-colors"
                >
                  <p className="truncate font-medium">
                    {path.split("/").pop()}
                  </p>
                  <p className="truncate text-xs text-muted-foreground">
                    {path}
                  </p>
                </button>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
