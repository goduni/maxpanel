import { AxiosError } from "axios";
import type { ApiErrorResponse } from "./api-types";

export function extractApiError(err: unknown, fallback: string): string {
  if (err instanceof AxiosError) {
    const data = err.response?.data;
    if (typeof data === "object" && data !== null && "error" in data) {
      const apiErr = data as ApiErrorResponse;
      const msg = apiErr.error?.message ?? fallback;
      if (apiErr.error?.upstream) {
        return `${msg}\n${JSON.stringify(apiErr.error.upstream, null, 2)}`;
      }
      return msg;
    }
  }
  return fallback;
}

export function extractApiErrorDetails(
  err: unknown,
): {
  message: string;
  details?: Array<{ field: string; message: string }>;
} | null {
  if (!(err instanceof AxiosError)) return null;
  const data = err.response?.data;
  if (typeof data !== "object" || data === null || !("error" in data))
    return null;
  const apiErr = data as ApiErrorResponse;
  return {
    message: apiErr.error?.message ?? "",
    details: apiErr.error?.details,
  };
}
