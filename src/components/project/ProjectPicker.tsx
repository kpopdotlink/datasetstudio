import { useState, useEffect } from 'react';
import { FolderOpen, Plus, Trash2, Clock, ChevronRight } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { useTranslation } from 'react-i18next';
import {
  getRecentProjects,
  removeRecentProject,
  createProject,
  openProject,
} from '../../api/project';
import { RecentProject } from '../../types';
import { useProjectStore } from '../../stores/projectStore';

export function ProjectPicker() {
  const { t, i18n } = useTranslation();
  const [recentProjects, setRecentProjects] = useState<RecentProject[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const { setProject, setIsLoading: setProjectLoading } = useProjectStore();

  useEffect(() => {
    loadRecentProjects();
  }, []);

  const loadRecentProjects = async () => {
    try {
      setIsLoading(true);
      const projects = await getRecentProjects();
      setRecentProjects(projects);
    } catch (err) {
      console.error('최근 프로젝트 로드 실패:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreateProject = async () => {
    try {
      const selected = await open({
        directory: true,
        title: '새 프로젝트 폴더 선택',
      });

      if (selected) {
        const name = prompt('프로젝트 이름을 입력하세요:');
        if (name) {
          setProjectLoading(true);
          const project = await createProject(selected, name);
          setProject(project);
        }
      }
    } catch (err) {
      setError(`프로젝트 생성 실패: ${err}`);
    } finally {
      setProjectLoading(false);
    }
  };

  const handleOpenProject = async () => {
    try {
      const selected = await open({
        directory: true,
        title: '프로젝트 폴더 선택',
      });

      if (selected) {
        setProjectLoading(true);
        const project = await openProject(selected);
        setProject(project);
      }
    } catch (err) {
      setError(`프로젝트 열기 실패: ${err}`);
    } finally {
      setProjectLoading(false);
    }
  };

  const handleOpenRecentProject = async (path: string) => {
    try {
      setProjectLoading(true);
      const project = await openProject(path);
      setProject(project);
    } catch (err) {
      setError(`프로젝트 열기 실패: ${err}`);
    } finally {
      setProjectLoading(false);
    }
  };

  const handleRemoveRecentProject = async (
    e: React.MouseEvent,
    path: string
  ) => {
    e.stopPropagation();
    try {
      await removeRecentProject(path);
      await loadRecentProjects();
    } catch (err) {
      console.error('최근 프로젝트 제거 실패:', err);
    }
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleDateString(i18n.language === 'ko' ? 'ko-KR' : 'en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center p-8">
      <div className="max-w-2xl w-full">
        {/* Header */}
        <div className="text-center mb-12">
          <h1 className="text-4xl font-bold text-gray-900 dark:text-white mb-2">
            {t("project.title")}
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            {t("project.subtitle")}
          </p>
        </div>

        {/* Error Message */}
        {error && (
          <div className="mb-6 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-700 dark:text-red-300">
            {error}
            <button
              onClick={() => setError(null)}
              className="ml-2 underline"
            >
              {t("common.close")}
            </button>
          </div>
        )}

        {/* Action Buttons */}
        <div className="grid grid-cols-2 gap-4 mb-8">
          <button
            onClick={handleCreateProject}
            className="flex items-center justify-center gap-3 p-6 bg-blue-600 hover:bg-blue-700 text-white rounded-xl transition-colors"
          >
            <Plus className="w-6 h-6" />
            <span className="font-medium">{t("project.create")}</span>
          </button>
          <button
            onClick={handleOpenProject}
            className="flex items-center justify-center gap-3 p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 text-gray-900 dark:text-white rounded-xl hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
          >
            <FolderOpen className="w-6 h-6" />
            <span className="font-medium">{t("project.open")}</span>
          </button>
        </div>

        {/* Recent Projects */}
        <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden">
          <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
            <h2 className="font-semibold text-gray-900 dark:text-white flex items-center gap-2">
              <Clock className="w-5 h-5" />
              {t("project.recent")}
            </h2>
          </div>

          {isLoading ? (
            <div className="p-8 text-center text-gray-500 dark:text-gray-400">
              {t("common.loading")}
            </div>
          ) : recentProjects.length === 0 ? (
            <div className="p-8 text-center text-gray-500 dark:text-gray-400">
              {t("project.noRecent")}
              <br />
              {t("project.noRecentHint")}
            </div>
          ) : (
            <ul className="divide-y divide-gray-200 dark:divide-gray-700">
              {recentProjects.map((project) => (
                <li
                  key={project.path}
                  onClick={() => handleOpenRecentProject(project.path)}
                  className="flex items-center justify-between px-6 py-4 hover:bg-gray-50 dark:hover:bg-gray-700/50 cursor-pointer transition-colors group"
                >
                  <div className="flex-1 min-w-0">
                    <p className="font-medium text-gray-900 dark:text-white truncate">
                      {project.name}
                    </p>
                    <p className="text-sm text-gray-500 dark:text-gray-400 truncate">
                      {project.path}
                    </p>
                    <p className="text-xs text-gray-400 dark:text-gray-500 mt-1">
                      {formatDate(project.last_opened)}
                    </p>
                  </div>
                  <div className="flex items-center gap-2 ml-4">
                    <button
                      onClick={(e) =>
                        handleRemoveRecentProject(e, project.path)
                      }
                      className="p-2 text-gray-400 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity"
                      title={t("project.removeFromList")}
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                    <ChevronRight className="w-5 h-5 text-gray-400" />
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </div>
  );
}
