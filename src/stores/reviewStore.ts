import { create } from "zustand";
import { ChunkDetail, ReviewFilters, ReviewQueue } from "../types";
import * as api from "../api/review";

interface ReviewState {
  queue: ReviewQueue | null;
  currentChunk: ChunkDetail | null;
  currentIndex: number;
  filters: ReviewFilters;
  isLoading: boolean;
  error: string | null;

  // Actions
  loadQueue: (filters?: ReviewFilters) => Promise<void>;
  loadChunkDetail: (chunkId: number) => Promise<void>;
  setStatus: (
    chunkId: number,
    status: "approved" | "skipped" | "rejected"
  ) => Promise<void>;
  saveEdit: (chunkId: number, text: string) => Promise<void>;
  next: () => Promise<void>;
  prev: () => Promise<void>;
  setFilters: (filters: ReviewFilters) => void;
}

export const useReviewStore = create<ReviewState>((set, get) => ({
  queue: null,
  currentChunk: null,
  currentIndex: 0,
  filters: { status: "pending" },
  isLoading: false,
  error: null,

  loadQueue: async (filters) => {
    const f = filters || get().filters;
    set({ isLoading: true, error: null, filters: f });
    try {
      const queue = await api.getReviewQueue(f);
      set({ queue, isLoading: false, currentIndex: 0 });

      if (queue.chunks.length > 0) {
        await get().loadChunkDetail(queue.chunks[0].id);
      } else {
        set({ currentChunk: null });
      }
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  loadChunkDetail: async (chunkId) => {
    try {
      const detail = await api.getChunkDetail(chunkId);
      set({ currentChunk: detail });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  setStatus: async (chunkId, status) => {
    try {
      await api.setReviewStatus(chunkId, status);

      // 큐 업데이트
      const queue = get().queue;
      if (queue) {
        const chunks = queue.chunks.map((c) =>
          c.id === chunkId ? { ...c, review_status: status } : c
        );
        set({
          queue: {
            ...queue,
            chunks,
            approved:
              status === "approved" ? queue.approved + 1 : queue.approved,
            skipped: status === "skipped" ? queue.skipped + 1 : queue.skipped,
            pending: queue.pending - 1,
          },
        });
      }
    } catch (e) {
      set({ error: String(e) });
    }
  },

  saveEdit: async (chunkId, text) => {
    try {
      await api.saveChunkEdit(chunkId, text);
      const current = get().currentChunk;
      if (current && current.chunk.id === chunkId) {
        set({ currentChunk: { ...current, edited_text: text } });
      }
    } catch (e) {
      set({ error: String(e) });
    }
  },

  next: async () => {
    const { queue, currentIndex } = get();
    if (!queue) return;

    const nextIndex = currentIndex + 1;
    if (nextIndex < queue.chunks.length) {
      set({ currentIndex: nextIndex });
      await get().loadChunkDetail(queue.chunks[nextIndex].id);
    }
  },

  prev: async () => {
    const { queue, currentIndex } = get();
    if (!queue) return;

    const prevIndex = currentIndex - 1;
    if (prevIndex >= 0) {
      set({ currentIndex: prevIndex });
      await get().loadChunkDetail(queue.chunks[prevIndex].id);
    }
  },

  setFilters: (filters) => {
    set({ filters });
  },
}));
