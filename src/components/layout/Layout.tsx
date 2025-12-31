import { Outlet } from "react-router-dom";
import Sidebar from "./Sidebar";
import Header from "./Header";
import { useJobStore } from "../../stores/jobStore";
import { useEffect } from "react";

export default function Layout() {
  const { subscribe, activeJobs } = useJobStore();

  useEffect(() => {
    const unsubscribe = subscribe();
    return () => {
      unsubscribe.then((fn) => fn());
    };
  }, [subscribe]);

  return (
    <div className="flex h-screen overflow-hidden bg-background">
      <Sidebar />
      <div className="flex flex-1 flex-col overflow-hidden">
        <Header />
        <main className="flex-1 overflow-auto p-6">
          <Outlet />
        </main>
        {activeJobs.length > 0 && (
          <div className="border-t border-border bg-card p-2">
            {activeJobs.map((job) => (
              <div key={job.id} className="flex items-center gap-2 text-sm">
                <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-secondary">
                  <div
                    className="h-full bg-primary transition-all"
                    style={{ width: `${job.progress * 100}%` }}
                  />
                </div>
                <span className="text-muted-foreground">
                  {job.message || `${Math.round(job.progress * 100)}%`}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
