import { invoke } from "@tauri-apps/api/core";
import { ChunkDetail, ReviewFilters, ReviewQueue } from "../types";

export async function getReviewQueue(
  filters: ReviewFilters
): Promise<ReviewQueue> {
  return invoke("get_review_queue", { filters });
}

export async function getChunkDetail(chunkId: number): Promise<ChunkDetail> {
  return invoke("get_chunk_detail", { chunkId });
}

export async function setReviewStatus(
  chunkId: number,
  status: string
): Promise<void> {
  return invoke("set_review_status", { chunkId, status });
}

export async function saveChunkEdit(
  chunkId: number,
  editedText: string
): Promise<void> {
  return invoke("save_chunk_edit", { chunkId, editedText });
}

export interface SearchResult {
  chunks: import("../types").Chunk[];
  total: number;
}

export async function searchChunks(
  query: string,
  limit?: number,
  offset?: number
): Promise<SearchResult> {
  return invoke("search_chunks", { query, limit, offset });
}

// 품질 검사 관련 타입
export interface QualityRules {
  min_chars?: number;
  max_chars?: number;
  min_tokens?: number;
  max_tokens?: number;
  max_consecutive_spaces?: number;
  max_consecutive_newlines?: number;
  max_special_char_ratio?: number;
  max_repeated_char_ratio?: number;
}

export interface QualityWarning {
  code: string;
  message: string;
  severity: "Info" | "Warning" | "Error";
}

export interface QualityCheckResult {
  warnings: QualityWarning[];
  score: number;
}

export async function checkChunkQuality(
  chunkId: number,
  rules?: QualityRules
): Promise<QualityCheckResult> {
  return invoke("check_chunk_quality", { chunkId, rules });
}

export async function getDefaultQualityRules(): Promise<QualityRules> {
  return invoke("get_default_quality_rules");
}

// 일괄 리뷰 필터
export interface BulkReviewFilter {
  document_ids?: number[];
  source_ids?: number[];
  current_status?: string;
}

export async function bulkSetReviewStatus(
  filter: BulkReviewFilter,
  status: string
): Promise<number> {
  return invoke("bulk_set_review_status", { filter, status });
}
