import api from "@/lib/api-client";
import { AxiosError } from "axios";
import type {
  ApiKey,
  ApiKeyCreateResponse,
  Bot,
  CreateApiKeyRequest,
  CreateBotRequest,
  OkResponse,
  PaginatedResponse,
  PaginationParams,
  RawProxyRequest,
  UpdateBotRequest,
} from "@/lib/api-types";

// Nested endpoints (through org/project)
export async function listBots(
  orgSlug: string,
  projectSlug: string,
  params?: PaginationParams,
): Promise<PaginatedResponse<Bot>> {
  const res = await api.get<PaginatedResponse<Bot>>(
    `/organizations/${orgSlug}/projects/${projectSlug}/bots`,
    { params },
  );
  return res.data;
}

export async function getBot(
  orgSlug: string,
  projectSlug: string,
  botId: string,
): Promise<Bot> {
  const res = await api.get<Bot>(
    `/organizations/${orgSlug}/projects/${projectSlug}/bots/${botId}`,
  );
  return res.data;
}

export async function createBot(
  orgSlug: string,
  projectSlug: string,
  data: CreateBotRequest,
): Promise<Bot> {
  const res = await api.post<Bot>(
    `/organizations/${orgSlug}/projects/${projectSlug}/bots`,
    data,
  );
  return res.data;
}

export async function updateBot(
  orgSlug: string,
  projectSlug: string,
  botId: string,
  data: UpdateBotRequest,
): Promise<Bot> {
  const res = await api.patch<Bot>(
    `/organizations/${orgSlug}/projects/${projectSlug}/bots/${botId}`,
    data,
  );
  return res.data;
}

export async function deleteBot(
  orgSlug: string,
  projectSlug: string,
  botId: string,
): Promise<OkResponse> {
  const res = await api.delete<OkResponse>(
    `/organizations/${orgSlug}/projects/${projectSlug}/bots/${botId}`,
  );
  return res.data;
}

// Flat endpoints (by bot ID)
export async function startBot(botId: string): Promise<OkResponse> {
  const res = await api.post<OkResponse>(`/bots/${botId}/start`);
  return res.data;
}

export async function stopBot(botId: string): Promise<OkResponse> {
  const res = await api.post<OkResponse>(`/bots/${botId}/stop`);
  return res.data;
}

export async function verifyBot(
  botId: string,
): Promise<Record<string, unknown>> {
  const res = await api.post<Record<string, unknown>>(
    `/bots/${botId}/verify`,
  );
  return res.data;
}

// API Keys
export async function listApiKeys(botId: string): Promise<ApiKey[]> {
  const res = await api.get<ApiKey[]>(`/bots/${botId}/api-keys`);
  return res.data;
}

export async function createApiKey(
  botId: string,
  data: CreateApiKeyRequest,
): Promise<ApiKeyCreateResponse> {
  const res = await api.post<ApiKeyCreateResponse>(
    `/bots/${botId}/api-keys`,
    data,
  );
  return res.data;
}

export async function deleteApiKey(
  botId: string,
  keyId: string,
): Promise<void> {
  await api.delete(`/bots/${botId}/api-keys/${keyId}`);
}

export async function proxyMaxApi(
  botId: string,
  data: RawProxyRequest,
): Promise<{ status: number; data: unknown; duration: number }> {
  const start = performance.now();
  try {
    const res = await api.post(`/bots/${botId}/max`, data);
    const duration = Math.round(performance.now() - start);
    return { status: res.status, data: res.data, duration };
  } catch (err) {
    const duration = Math.round(performance.now() - start);
    if (err instanceof AxiosError && err.response) {
      return { status: err.response.status, data: err.response.data, duration };
    }
    throw err;
  }
}
