import { invoke } from "@tauri-apps/api/core";
import { Document, DocumentFilters, ScanResult, Source } from "../types";

export async function addSource(
  sourceType: string,
  pathOrContent: string,
  displayName?: string
): Promise<Source> {
  return invoke("add_source", {
    sourceType,
    pathOrContent,
    displayName,
  });
}

export async function scanSources(): Promise<ScanResult> {
  return invoke("scan_sources");
}

export async function listDocuments(
  filters: DocumentFilters
): Promise<Document[]> {
  return invoke("list_documents", { filters });
}

export async function removeDocument(docId: number): Promise<void> {
  return invoke("remove_document", { docId });
}

export async function listSources(): Promise<Source[]> {
  return invoke("list_sources");
}

export async function removeSource(sourceId: number): Promise<void> {
  return invoke("remove_source", { sourceId });
}

export async function rescanSource(sourceId: number): Promise<ScanResult> {
  return invoke("rescan_source", { sourceId });
}

export async function getManualDocumentText(docId: number): Promise<string> {
  return invoke("get_manual_document_text", { docId });
}

export interface DocumentChangeInfo {
  document_id: number;
  display_name: string;
  original_path: string;
  old_checksum: string;
  new_checksum: string;
  change_type: "modified" | "deleted";
}

export interface ChangeDetectionResult {
  changes: DocumentChangeInfo[];
  total_checked: number;
}

export async function checkDocumentChanges(): Promise<ChangeDetectionResult> {
  return invoke("check_document_changes");
}

export async function resolveDocumentChange(
  documentId: number,
  action: "rechunk" | "update_checksum" | "remove"
): Promise<void> {
  return invoke("resolve_document_change", { documentId, action });
}
