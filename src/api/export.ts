import { invoke } from "@tauri-apps/api/core";
import { ExportFilters, ExportRun, Preset } from "../types";

export interface UploadCommand {
  platform: string;
  name: string;
  command: string;
  description: string;
}

export async function listPresets(): Promise<Preset[]> {
  return invoke("list_presets");
}

export async function startExportJob(
  presetId: number,
  filters: ExportFilters,
  maxFileSizeMb?: number
): Promise<number> {
  return invoke("start_export_job", { presetId, filters, maxFileSizeMb });
}

export async function listExportRuns(): Promise<ExportRun[]> {
  return invoke("list_export_runs");
}

export async function openExportFolder(): Promise<void> {
  return invoke("open_export_folder");
}

export async function generateUploadCommands(
  exportId: string,
  datasetName: string
): Promise<UploadCommand[]> {
  return invoke("generate_upload_commands", { exportId, datasetName });
}

export async function createPreset(
  name: string,
  mappingJson: string,
  validatorsJson?: string
): Promise<Preset> {
  return invoke("create_preset", { name, mappingJson, validatorsJson });
}

export async function updatePreset(
  presetId: number,
  name: string,
  mappingJson: string,
  validatorsJson?: string
): Promise<Preset> {
  return invoke("update_preset", { presetId, name, mappingJson, validatorsJson });
}

export async function deletePreset(presetId: number): Promise<void> {
  return invoke("delete_preset", { presetId });
}
