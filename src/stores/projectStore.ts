import { create } from "zustand";
import { ProjectInfo, ProjectStats } from "../types";
import * as api from "../api/project";

interface ProjectState {
  isOpen: boolean;
  projectInfo: ProjectInfo | null;
  stats: ProjectStats | null;
  recentProjects: string[];
  isLoading: boolean;
  error: string | null;

  // Actions
  createProject: (path: string, name: string) => Promise<void>;
  openProject: (path: string) => Promise<void>;
  closeProject: () => Promise<void>;
  refreshInfo: () => Promise<void>;
  refreshStats: () => Promise<void>;
  addRecentProject: (path: string) => void;
  setProject: (info: ProjectInfo) => void;
  setIsLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

export const useProjectStore = create<ProjectState>((set, get) => ({
  isOpen: false,
  projectInfo: null,
  stats: null,
  recentProjects: JSON.parse(
    localStorage.getItem("recent-projects") || "[]"
  ) as string[],
  isLoading: false,
  error: null,

  createProject: async (path, name) => {
    set({ isLoading: true, error: null });
    try {
      const info = await api.createProject(path, name);
      set({ projectInfo: info, isOpen: true, isLoading: false });
      get().addRecentProject(path);
      await get().refreshStats();
    } catch (e) {
      set({ error: String(e), isLoading: false });
      throw e;
    }
  },

  openProject: async (path) => {
    set({ isLoading: true, error: null });
    try {
      const info = await api.openProject(path);
      set({ projectInfo: info, isOpen: true, isLoading: false });
      get().addRecentProject(path);
      await get().refreshStats();
    } catch (e) {
      set({ error: String(e), isLoading: false });
      throw e;
    }
  },

  closeProject: async () => {
    try {
      await api.closeProject();
      set({ projectInfo: null, stats: null, isOpen: false });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  refreshInfo: async () => {
    if (!get().isOpen) return;
    try {
      const info = await api.getProjectInfo();
      set({ projectInfo: info });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  refreshStats: async () => {
    if (!get().isOpen) return;
    try {
      const stats = await api.getProjectStats();
      set({ stats });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  addRecentProject: (path) => {
    const recent = get().recentProjects.filter((p) => p !== path);
    recent.unshift(path);
    const updated = recent.slice(0, 10);
    localStorage.setItem("recent-projects", JSON.stringify(updated));
    set({ recentProjects: updated });
  },

  setProject: (info) => {
    set({ projectInfo: info, isOpen: true });
    get().refreshStats();
  },

  setIsLoading: (loading) => {
    set({ isLoading: loading });
  },

  setError: (error) => {
    set({ error });
  },
}));
