import { invoke } from "@tauri-apps/api/core";
import { Preset } from "../types";

export interface ManualEntry {
  id: number;
  preset_id: number;
  data_json: string;
  review_status: string;
  created_at: string;
  updated_at: string;
}

export interface ManualEntryFilters {
  preset_id?: number;
  status?: string;
  limit?: number;
  offset?: number;
}

export interface ManualEntryListResult {
  entries: ManualEntry[];
  total: number;
  pending: number;
  approved: number;
  rejected: number;
  preset: Preset | null;
}

export async function createManualEntry(
  presetId: number,
  dataJson: string
): Promise<ManualEntry> {
  return invoke("create_manual_entry", { presetId, dataJson });
}

export async function listManualEntries(
  filters: ManualEntryFilters
): Promise<ManualEntryListResult> {
  return invoke("list_manual_entries", { filters });
}

export async function updateManualEntry(
  entryId: number,
  dataJson: string
): Promise<ManualEntry> {
  return invoke("update_manual_entry", { entryId, dataJson });
}

export async function setManualEntryStatus(
  entryId: number,
  status: string
): Promise<void> {
  return invoke("set_manual_entry_status", { entryId, status });
}

export async function deleteManualEntry(entryId: number): Promise<void> {
  return invoke("delete_manual_entry", { entryId });
}

export async function importJsonl(
  presetId: number,
  filePath: string
): Promise<number> {
  return invoke("import_jsonl", { presetId, filePath });
}

export async function getPresetFields(presetId: number): Promise<string[]> {
  return invoke("get_preset_fields", { presetId });
}
