// 프로젝트 타입
export interface ProjectSettings {
  chunk_params: ChunkParams;
  default_export_format: string;
  max_export_file_size_mb: number;
}

export interface ProjectMeta {
  name: string;
  version: string;
  created_at: string;
  updated_at: string;
  settings: ProjectSettings;
}

export interface ProjectInfo {
  path: string;
  meta: ProjectMeta;
  document_count: number;
  chunk_count: number;
  approved_count: number;
}

export interface RecentProject {
  path: string;
  name: string;
  last_opened: string;
}

// 소스 타입
export type SourceType = "folder" | "file" | "manual";

export interface Source {
  id: number;
  type: SourceType;
  path_or_key: string;
  display_name?: string;
  added_at: string;
  meta_json?: string;
}

// 문서 타입
export type DocumentStatus =
  | "unprocessed"
  | "chunked"
  | "in_review"
  | "approved"
  | "exported"
  | "needs_attention";

export interface Document {
  id: number;
  source_id: number;
  display_name: string;
  original_path?: string;
  checksum: string;
  encoding: string;
  status: DocumentStatus;
  byte_size: number;
  created_at: string;
  updated_at: string;
}

export interface DocumentFilters {
  status?: string;
  source_id?: number;
  search?: string;
  limit?: number;
  offset?: number;
}

// 청크 타입
export type ReviewStatus = "pending" | "approved" | "skipped" | "rejected";
export type LengthUnit = "Character" | "Token";

export interface ChunkParams {
  max_len: number;
  overlap_len: number;
  length_unit: LengthUnit;
  preserve_paragraph: boolean;
  preserve_sentence: boolean;
  allow_hard_cut: boolean;
}

export interface Chunk {
  id: number;
  document_id: number;
  chunk_version_id: number;
  chunk_index: number;
  start_offset: number;
  end_offset: number;
  text_cached?: string;
  char_count: number;
  token_est: number;
  overlap_prev: number;
  overlap_next: number;
  review_status: ReviewStatus;
  is_hard_cut: boolean;
  created_at: string;
  updated_at: string;
}

export interface ChunkPreview {
  index: number;
  text: string;
  char_count: number;
  token_est: number;
  overlap_prev: number;
  overlap_next: number;
  is_hard_cut: boolean;
}

export interface ChunkDetail {
  chunk: Chunk;
  original_text: string;
  edited_text?: string;
  document_name: string;
  prev_chunk_id?: number;
  next_chunk_id?: number;
}

export interface ReviewFilters {
  status?: string;
  document_id?: number;
  source_id?: number;
  limit?: number;
  offset?: number;
}

export interface ReviewQueue {
  chunks: Chunk[];
  total: number;
  pending: number;
  approved: number;
  skipped: number;
}

// Export 타입
export interface Preset {
  id: number;
  name: string;
  schema_version: string;
  mapping_json: string;
  validators_json?: string;
  is_builtin: boolean;
  created_at: string;
}

export interface ExportFilters {
  approved_only: boolean;
  document_ids?: number[];
  source_ids?: number[];
  tags?: string[];
  include_manual_entries?: boolean;
  split_by_source?: boolean;
}

export interface ExportRun {
  id: number;
  preset_id?: number;
  preset_name: string;
  params_json: string;
  filters_json: string;
  stats_json?: string;
  created_at: string;
}

// 잡 타입
export type JobStatus =
  | "pending"
  | "running"
  | "completed"
  | "failed"
  | "cancelled";

export interface Job {
  id: number;
  type: string;
  status: JobStatus;
  progress: number;
  payload_json?: string;
  error_message?: string;
  created_at: string;
  updated_at: string;
}

export interface JobEvent {
  job_id: number;
  event_type:
    | { type: "progress"; progress: number; message?: string }
    | { type: "completed"; result?: string }
    | { type: "failed"; error: string };
}

// 통계 타입
export interface ProjectStats {
  document_count: number;
  chunk_count: number;
  pending_count: number;
  approved_count: number;
  skipped_count: number;
  rejected_count: number;
  export_count: number;
  total_chars: number;
  total_tokens_est: number;
  avg_chunk_length: number;
  approval_rate: number;
}

// 스캔 결과
export interface ScanResult {
  total_files: number;
  new_files: number;
  skipped_files: number;
  errors: string[];
}
