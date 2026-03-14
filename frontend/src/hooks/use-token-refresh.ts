import { useState, useEffect } from "react";
import axios from "axios";
import { useAuthStore } from "@/stores/auth";

export function useTokenRefresh() {
  const [refreshing, setRefreshing] = useState(false);
  const { accessToken, refreshToken, setTokens, clearTokens } = useAuthStore();
  const needsRefresh = !accessToken && !!refreshToken;

  useEffect(() => {
    if (!needsRefresh) return;
    setRefreshing(true);
    axios
      .post("/api/auth/refresh", { refresh_token: refreshToken })
      .then((res) => {
        setTokens(res.data.access_token, res.data.refresh_token);
      })
      .catch(() => {
        clearTokens();
      })
      .finally(() => setRefreshing(false));
    // eslint-disable-next-line react-hooks/exhaustive-deps -- only needsRefresh triggers; refreshToken/setTokens/clearTokens are stable Zustand selectors
  }, [needsRefresh]);

  return { refreshing, needsRefresh, accessToken, refreshToken };
}
