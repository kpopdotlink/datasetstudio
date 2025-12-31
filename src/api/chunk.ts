import { invoke } from "@tauri-apps/api/core";
import { ChunkParams, ChunkPreview } from "../types";

export interface ChunkFilter {
  document_ids?: number[];
  source_ids?: number[];
  rechunk?: boolean;
}

export async function startChunkJob(
  params: ChunkParams,
  filter?: ChunkFilter
): Promise<number> {
  return invoke("start_chunk_job", { params, filter });
}

export async function getChunkPreview(
  docId: number,
  params: ChunkParams,
  sampleSize: number
): Promise<ChunkPreview[]> {
  return invoke("get_chunk_preview", { docId, params, sampleSize });
}

export async function cancelJob(jobId: number): Promise<void> {
  return invoke("cancel_job", { jobId });
}

export interface MergedChunk {
  id: number;
  text: string;
  char_count: number;
  token_est: number;
}

export interface SplitChunks {
  first: MergedChunk;
  second: MergedChunk;
}

export async function mergeChunks(
  chunkId1: number,
  chunkId2: number
): Promise<MergedChunk> {
  return invoke("merge_chunks", { chunkId1, chunkId2 });
}

export async function splitChunk(
  chunkId: number,
  splitPosition: number
): Promise<SplitChunks> {
  return invoke("split_chunk", { chunkId, splitPosition });
}
