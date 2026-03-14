import axios, {
  type AxiosError,
  type InternalAxiosRequestConfig,
} from "axios";
import { useAuthStore } from "@/stores/auth";
import type { AuthTokens, RefreshRequest } from "@/lib/api-types";

const api = axios.create({
  baseURL: "/api",
  headers: { "Content-Type": "application/json" },
});

// Request interceptor: attach access token
api.interceptors.request.use((config) => {
  const { accessToken } = useAuthStore.getState();
  if (accessToken) {
    config.headers.Authorization = `Bearer ${accessToken}`;
  }
  return config;
});

// Response interceptor: handle 429 rate limiting
api.interceptors.response.use(undefined, (error: AxiosError) => {
  if (error.response?.status === 429) {
    const retryAfter = parseInt(
      error.response.headers["retry-after"] ?? "10",
      10,
    );
    // Dispatch custom event so UI components can show toast
    window.dispatchEvent(
      new CustomEvent("maxpanel:rate-limited", { detail: { retryAfter } }),
    );
  }
  return Promise.reject(error);
});

// Response interceptor: handle 401 with token refresh
let isRefreshing = false;
let pendingRequests: Array<{
  resolve: () => void;
  reject: (error: unknown) => void;
}> = [];

function processPendingRequests(error: unknown | null) {
  pendingRequests.forEach(({ resolve, reject }) => {
    if (error) {
      reject(error);
    } else {
      resolve();
    }
  });
  pendingRequests = [];
}

api.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    const originalRequest = error.config as InternalAxiosRequestConfig & {
      _retry?: boolean;
    };

    // Only handle 401s, not on auth endpoints themselves.
    // Note: originalRequest.url is relative to baseURL ("/api"),
    // so values are like "/auth/login", "/auth/refresh", etc.
    if (
      error.response?.status !== 401 ||
      originalRequest._retry ||
      originalRequest.url?.startsWith("/auth/")
    ) {
      return Promise.reject(error);
    }

    originalRequest._retry = true;

    if (isRefreshing) {
      // Reject if too many requests are already queued
      if (pendingRequests.length >= 100) {
        return Promise.reject(error);
      }
      // Queue this request until refresh completes
      return new Promise((resolve, reject) => {
        pendingRequests.push({
          resolve: () => {
            const { accessToken } = useAuthStore.getState();
            originalRequest.headers.Authorization = `Bearer ${accessToken}`;
            resolve(api(originalRequest));
          },
          reject,
        });
      });
    }

    isRefreshing = true;
    const { refreshToken, setTokens, clearTokens } = useAuthStore.getState();

    if (!refreshToken) {
      clearTokens();
      return Promise.reject(error);
    }

    try {
      const { data } = await axios.post<AuthTokens>(
        "/api/auth/refresh",
        { refresh_token: refreshToken } satisfies RefreshRequest,
        { headers: { "Content-Type": "application/json" } },
      );

      const { access_token, refresh_token } = data;
      if (
        typeof access_token === "string" &&
        access_token.length > 0 &&
        typeof refresh_token === "string" &&
        refresh_token.length > 0
      ) {
        setTokens(access_token, refresh_token);
      } else {
        clearTokens();
        processPendingRequests(new Error("Invalid token format"));
        return Promise.reject(new Error("Invalid token format"));
      }
      originalRequest.headers.Authorization = `Bearer ${access_token}`;
      processPendingRequests(null);
      return api(originalRequest);
    } catch (refreshError) {
      clearTokens();
      processPendingRequests(refreshError);
      return Promise.reject(refreshError);
    } finally {
      // Note: isRefreshing is reset after processPendingRequests. If a retried
      // request immediately 401s again, it could be queued but never drained.
      // This is acceptable because a fresh token should not expire immediately.
      isRefreshing = false;
    }
  },
);

export default api;
