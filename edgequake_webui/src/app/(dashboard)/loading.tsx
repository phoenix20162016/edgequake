/**
 * Route-level loading UI for dashboard home (`/`).
 * SPEC-144 Instant Navigations shell for home ↔ documents transitions.
 */
import { Skeleton } from "@/components/ui/skeleton";

export default function DashboardHomeLoading() {
  return (
    <div
      className="flex h-full min-h-0 flex-col gap-4 p-4"
      role="status"
      aria-busy="true"
      aria-label="Loading dashboard"
      data-testid="dashboard-route-loading"
    >
      <Skeleton className="h-8 w-48" />
      <div className="grid min-w-0 gap-3 sm:grid-cols-2 lg:grid-cols-4 [&>*]:min-w-0">
        {[...Array(4)].map((_, i) => (
          <Skeleton key={i} className="h-24 w-full rounded-md" />
        ))}
      </div>
      <Skeleton className="h-40 w-full rounded-md" />
    </div>
  );
}
