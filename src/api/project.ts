import { invoke } from "@tauri-apps/api/core";
import { ProjectInfo, ProjectStats, ProjectSettings, RecentProject } from "../types";

export async function createProject(
  path: string,
  name: string
): Promise<ProjectInfo> {
  return invoke("create_project", { path, name });
}

export async function openProject(path: string): Promise<ProjectInfo> {
  return invoke("open_project", { path });
}

export async function closeProject(): Promise<void> {
  return invoke("close_project");
}

export async function getProjectInfo(): Promise<ProjectInfo> {
  return invoke("get_project_info");
}

export async function getProjectStats(): Promise<ProjectStats> {
  return invoke("get_project_stats");
}

export async function getRecentProjects(): Promise<RecentProject[]> {
  return invoke("get_recent_projects");
}

export async function removeRecentProject(path: string): Promise<void> {
  return invoke("remove_recent_project", { path });
}

export async function getProjectSettings(): Promise<ProjectSettings> {
  return invoke("get_project_settings");
}

export async function updateProjectSettings(
  settings: ProjectSettings
): Promise<void> {
  return invoke("update_project_settings", { settings });
}

export async function getAppTheme(): Promise<string> {
  return invoke("get_app_theme");
}

export async function setAppTheme(theme: string): Promise<void> {
  return invoke("set_app_theme", { theme });
}
