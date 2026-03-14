import { Navigate } from "react-router-dom";
import { useAuthStore } from "@/stores/auth";

export function GuestRoute({ children }: { children: React.ReactNode }) {
  const { accessToken, refreshToken } = useAuthStore();

  if (accessToken || refreshToken) {
    return <Navigate to="/" replace />;
  }

  return <>{children}</>;
}
