import { Navigate, useLocation } from "react-router-dom";
import { Skeleton } from "@/components/ui/skeleton";
import { useTokenRefresh } from "@/hooks/use-token-refresh";

export function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { refreshing, accessToken, refreshToken } = useTokenRefresh();
  const location = useLocation();

  // No tokens at all — redirect to login
  if (!accessToken && !refreshToken && !refreshing) {
    return <Navigate to="/login" state={{ from: location }} replace />;
  }

  // Refreshing or waiting for access token — block children from rendering
  if (!accessToken) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="space-y-4 w-48">
          <Skeleton className="h-4 w-full" />
          <Skeleton className="h-4 w-3/4" />
        </div>
      </div>
    );
  }

  return <>{children}</>;
}
