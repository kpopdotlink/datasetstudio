import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { JobEvent } from "../types";

interface ActiveJob {
  id: number;
  type: string;
  progress: number;
  message?: string;
}

interface JobState {
  activeJobs: ActiveJob[];
  isSubscribed: boolean;

  // Actions
  subscribe: () => Promise<() => void>;
  addJob: (id: number, type: string) => void;
  removeJob: (id: number) => void;
  updateProgress: (id: number, progress: number, message?: string) => void;
}

export const useJobStore = create<JobState>((set, get) => ({
  activeJobs: [],
  isSubscribed: false,

  subscribe: async () => {
    if (get().isSubscribed) {
      return () => {};
    }

    const unlistenProgress = await listen<JobEvent>("job-progress", (event) => {
      const { job_id, event_type } = event.payload;
      if (event_type.type === "progress") {
        get().updateProgress(job_id, event_type.progress, event_type.message);
      }
    });

    const unlistenCompleted = await listen<JobEvent>(
      "job-completed",
      (event) => {
        get().removeJob(event.payload.job_id);
      }
    );

    const unlistenFailed = await listen<JobEvent>("job-failed", (event) => {
      get().removeJob(event.payload.job_id);
    });

    set({ isSubscribed: true });

    return () => {
      unlistenProgress();
      unlistenCompleted();
      unlistenFailed();
      set({ isSubscribed: false });
    };
  },

  addJob: (id, type) => {
    set((state) => ({
      activeJobs: [...state.activeJobs, { id, type, progress: 0 }],
    }));
  },

  removeJob: (id) => {
    set((state) => ({
      activeJobs: state.activeJobs.filter((j) => j.id !== id),
    }));
  },

  updateProgress: (id, progress, message) => {
    set((state) => ({
      activeJobs: state.activeJobs.map((j) =>
        j.id === id ? { ...j, progress, message } : j
      ),
    }));
  },
}));
