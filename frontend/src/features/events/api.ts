import api from "@/lib/api-client";
import type {
  BotChat,
  BotEvent,
  CursorPaginatedResponse,
  CursorParams,
} from "@/lib/api-types";

export async function listEvents(
  botId: string,
  params?: CursorParams & { direction?: "inbound" | "outbound" },
): Promise<CursorPaginatedResponse<BotEvent>> {
  const res = await api.get<CursorPaginatedResponse<BotEvent>>(
    `/bots/${botId}/events`,
    { params },
  );
  return res.data;
}

export async function getEvent(
  botId: string,
  eventId: string,
  createdAt?: string,
): Promise<BotEvent> {
  const res = await api.get<BotEvent>(`/bots/${botId}/events/${eventId}`, {
    params: createdAt ? { created_at: createdAt } : undefined,
  });
  return res.data;
}

export async function listChats(
  botId: string,
  params?: CursorParams & { search?: string },
): Promise<CursorPaginatedResponse<BotChat>> {
  const res = await api.get<CursorPaginatedResponse<BotChat>>(
    `/bots/${botId}/chats`,
    { params },
  );
  return res.data;
}

export async function syncChats(
  botId: string,
): Promise<{ synced: number }> {
  const res = await api.post<{ synced: number }>(
    `/bots/${botId}/chats/sync`,
  );
  return res.data;
}

export async function syncChatHistory(
  botId: string,
  chatId: number,
): Promise<{ synced: number }> {
  const res = await api.post<{ synced: number }>(
    `/bots/${botId}/chats/${chatId}/sync-history`,
  );
  return res.data;
}

export interface HistoryMessage {
  sender?: { user_id: number; name?: string; first_name?: string; last_name?: string; is_bot?: boolean };
  recipient: { chat_id: number; chat_type: string };
  timestamp: number;
  body: { mid: string; seq: number; text?: string; attachments?: unknown[] };
  link?: unknown;
  stat?: { views: number };
}

export async function fetchChatHistory(
  botId: string,
  chatId: number,
  params?: { to?: number; count?: number },
): Promise<{ messages: HistoryMessage[] }> {
  const res = await api.get<{ messages: HistoryMessage[] }>(
    `/bots/${botId}/chats/${chatId}/history`,
    { params },
  );
  return res.data;
}

export async function listChatEvents(
  botId: string,
  chatId: number,
  params?: CursorParams,
): Promise<CursorPaginatedResponse<BotEvent>> {
  const res = await api.get<CursorPaginatedResponse<BotEvent>>(
    `/bots/${botId}/chats/${chatId}/events`,
    { params },
  );
  return res.data;
}
