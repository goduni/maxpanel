import { useInfiniteQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as eventsApi from "../api";

export function useEvents(
  botId: string,
  refetchInterval?: number | false,
  direction?: "inbound" | "outbound",
) {
  return useInfiniteQuery({
    queryKey: ["events", botId, { direction }],
    queryFn: ({ pageParam }) =>
      eventsApi.listEvents(botId, {
        cursor: pageParam ?? undefined,
        limit: 50,
        direction,
      }),
    initialPageParam: null as string | null,
    getNextPageParam: (lastPage) =>
      lastPage.pagination.has_more ? lastPage.pagination.next_cursor : undefined,
    enabled: !!botId,
    staleTime: 10 * 1000,
    refetchInterval: refetchInterval !== false ? refetchInterval : undefined,
  });
}

export function useChats(botId: string, search?: string) {
  return useInfiniteQuery({
    queryKey: ["chats", botId, search],
    queryFn: ({ pageParam }) =>
      eventsApi.listChats(botId, {
        cursor: pageParam ?? undefined,
        limit: 50,
        search: search || undefined,
      }),
    initialPageParam: null as string | null,
    getNextPageParam: (lastPage) =>
      lastPage.pagination.has_more ? lastPage.pagination.next_cursor : undefined,
    enabled: !!botId,
    staleTime: 10 * 1000,
  });
}

export function useSyncChats(botId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => eventsApi.syncChats(botId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["chats", botId] });
    },
  });
}

export function useChatEvents(botId: string, chatId: number | null) {
  return useInfiniteQuery({
    queryKey: ["chat-events", botId, chatId],
    queryFn: ({ pageParam }) =>
      eventsApi.listChatEvents(botId, chatId!, {
        cursor: pageParam ?? undefined,
        limit: 50,
      }),
    initialPageParam: null as string | null,
    getNextPageParam: (lastPage) =>
      lastPage.pagination.has_more ? lastPage.pagination.next_cursor : undefined,
    enabled: !!botId && chatId !== null,
    staleTime: 10 * 1000,
  });
}

export function useSyncChatHistory(botId: string, chatId: number) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => eventsApi.syncChatHistory(botId, chatId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["chat-events", botId, chatId] });
    },
  });
}

/** Convert millisecond timestamp to seconds (Max API uses seconds for from/to params). */
function msToSec(ts: number): number {
  return ts > 1_000_000_000_000 ? Math.floor(ts / 1000) : ts;
}

export function useChatHistory(
  botId: string,
  chatId: number | null,
  enabled: boolean,
  initialTo?: number,
) {
  // Convert initial timestamp to seconds for Max API, subtract 1s to exclude boundary
  const initialToSec = initialTo != null ? msToSec(initialTo) - 1 : undefined;

  return useInfiniteQuery({
    queryKey: ["chat-history", botId, chatId],
    queryFn: ({ pageParam }) =>
      eventsApi.fetchChatHistory(botId, chatId!, {
        to: pageParam,
        count: 50,
      }),
    initialPageParam: initialToSec,
    getNextPageParam: (lastPage, allPages) => {
      const msgs = lastPage.messages;
      // No more pages if empty or fewer than requested
      if (msgs.length < 50) return undefined;
      const lastTs = msgs[msgs.length - 1]?.timestamp;
      if (lastTs == null) return undefined;
      // Convert to seconds and subtract 1 to avoid overlap
      const nextTo = msToSec(lastTs) - 1;
      // Guard against infinite loop: if nextTo equals previous page param, stop
      if (allPages.length >= 2) {
        const prevLastTs = allPages[allPages.length - 2]?.messages?.slice(-1)[0]?.timestamp;
        if (prevLastTs != null && msToSec(prevLastTs) - 1 === nextTo) return undefined;
      }
      return nextTo;
    },
    enabled: !!botId && chatId !== null && enabled,
    staleTime: 60 * 1000,
  });
}
