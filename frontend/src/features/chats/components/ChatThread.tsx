import { memo, useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { ArrowLeft, Download, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { JsonViewer } from "@/components/ui/json-viewer";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { formatDate } from "@/lib/utils";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { proxyMaxApi } from "@/features/bots/api";
import { useChatEvents, useChatHistory, useSyncChatHistory } from "@/features/events/hooks/use-events";
import type { BotChat, BotEvent } from "@/lib/api-types";
import {
  getPayload,
  getMessage,
  getBody,
  getSender,
  getMid,
  isOutboundEvent,
} from "@/features/chats/lib/payload";
import { MessageBubble } from "./MessageBubble";
import { SystemEvent } from "./SystemEvent";
import { MessageComposer, type ReplyTo } from "./MessageComposer";

// Max API event types that represent chat messages
const MESSAGE_TYPES = new Set([
  "message_created",
  "message_edited",
  "message_removed",
  "message_sent",
]);

/** Extract the sender userId from an event, or null if not a message / no sender. */
function getSenderId(event: BotEvent): number | null {
  if (!MESSAGE_TYPES.has(event.update_type)) return null;
  // Outbound events are always from the bot
  if (isOutboundEvent(event)) return -1;
  const payload = getPayload(event);
  if (!payload) return null;
  const message = getMessage(payload, event);
  if (!message) return null;
  const sender = getSender(message, event);
  return sender?.userId ?? null;
}

/** Build a ReplyTo object from an event */
function buildReplyTo(event: BotEvent): ReplyTo | null {
  const payload = getPayload(event);
  if (!payload) return null;
  const message = getMessage(payload, event);
  if (!message) return null;
  const mid = getMid(message);
  if (!mid) return null;
  const sender = getSender(message, event);
  const body = getBody(message);
  const text =
    typeof body?.text === "string" ? body.text.slice(0, 100) : "";
  return {
    mid,
    senderName: sender?.name ?? "?",
    text,
  };
}

const EventRow = memo(function EventRow({
  event,
  showSender,
  onInspect,
  onReply,
  onForward,
  onDelete,
}: {
  event: BotEvent;
  showSender: boolean;
  onInspect: (event: BotEvent) => void;
  onReply: (event: BotEvent) => void;
  onForward: (event: BotEvent) => void;
  onDelete: (event: BotEvent) => void;
}) {
  const handleInspect = useCallback(() => onInspect(event), [onInspect, event]);
  const handleReply = useCallback(() => onReply(event), [onReply, event]);
  const handleForward = useCallback(() => onForward(event), [onForward, event]);
  const handleDelete = useCallback(() => onDelete(event), [onDelete, event]);

  if (MESSAGE_TYPES.has(event.update_type)) {
    return (
      <MessageBubble
        event={event}
        showSender={showSender}
        onInspect={handleInspect}
        onReply={handleReply}
        onForward={handleForward}
        onDelete={handleDelete}
      />
    );
  }
  return <SystemEvent event={event} onInspect={handleInspect} />;
});

export function ChatThread({
  botId,
  chat,
  onBack,
}: {
  botId: string;
  chat: BotChat;
  onBack?: () => void;
}) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [showChatInfo, setShowChatInfo] = useState(false);
  const { data, isLoading, fetchNextPage, hasNextPage, isFetchingNextPage } =
    useChatEvents(botId, chat.chat_id);
  const [inspectEvent, setInspectEvent] = useState<BotEvent | null>(null);
  const [replyTo, setReplyTo] = useState<ReplyTo | null>(null);

  // API returns newest-first — keep that order for flex-col-reverse (newest at visual bottom)
  const events = useMemo(
    () => data?.pages.flatMap((p) => p.data) ?? [],
    [data],
  );

  // When local events are exhausted (or empty), load history from Max API proxy.
  // For empty chats: no `to` param → Max API returns newest messages.
  // For non-empty chats: `to` = oldest local timestamp → only older messages.
  const localDone = hasNextPage === false && !isLoading;
  const oldestLocalTimestamp = localDone && events.length > 0
    ? events[events.length - 1]?.timestamp
    : undefined;

  const {
    data: historyData,
    fetchNextPage: fetchNextHistory,
    hasNextPage: hasMoreHistory,
    isFetchingNextPage: isFetchingHistory,
  } = useChatHistory(botId, chat.chat_id, localDone, oldestLocalTimestamp);

  // Minimum timestamp among local events — anything >= this is already displayed locally
  const minLocalTimestamp = useMemo(() => {
    if (!events.length) return Infinity;
    return Math.min(...events.map((e) => e.timestamp));
  }, [events]);

  // Convert history messages to event-like objects, filtering out overlaps with local events
  const historyEvents: BotEvent[] = useMemo(() => {
    if (!historyData) return [];
    return historyData.pages.flatMap((page) =>
      page.messages
        .filter((msg) => msg.timestamp < minLocalTimestamp)
        .map((msg) => {
        const sender = msg.sender as Record<string, unknown> | undefined;
        const isBot = sender?.is_bot === true;
        return {
          id: msg.body?.mid ?? `hist-${msg.timestamp}`,
          bot_id: botId,
          max_update_id: null,
          update_type: "message_created",
          chat_id: msg.recipient?.chat_id ?? chat.chat_id,
          sender_id: msg.sender?.user_id ?? null,
          timestamp: msg.timestamp,
          raw_payload: { message: msg, timestamp: msg.timestamp },
          created_at: new Date(msg.timestamp).toISOString(),
          direction: (isBot ? "outbound" : "inbound") as BotEvent["direction"],
          source: "history" as const,
        };
      }),
    );
  }, [historyData, botId, chat.chat_id]);

  // Merge local events and history events (stable reference when history is empty)
  const allEvents = useMemo(
    () => (historyEvents.length === 0 ? events : [...events, ...historyEvents]),
    [events, historyEvents],
  );

  // Pre-compute showSender flags in a single pass
  const showSenderFlags = useMemo(() => {
    const ids = allEvents.map((e) => getSenderId(e));
    return allEvents.map((_, i) =>
      i === allEvents.length - 1 ? true : ids[i] !== ids[i + 1],
    );
  }, [allEvents]);

  // Infinite scroll — sentinel at the top (oldest messages)
  // When local events exist, load those first. Only switch to history when exhausted.
  const sentinelRef = useInfiniteScroll({
    hasNextPage: hasNextPage || (localDone && !!hasMoreHistory),
    isFetchingNextPage: isFetchingNextPage || isFetchingHistory,
    fetchNextPage: hasNextPage ? fetchNextPage : fetchNextHistory,
  });

  const confirm = useConfirm();

  const handleReply = useCallback(
    (event: BotEvent) => {
      const reply = buildReplyTo(event);
      if (reply) setReplyTo(reply);
    },
    [],
  );

  const handleForward = useCallback(
    async (event: BotEvent) => {
      const payload = getPayload(event);
      const message = payload ? getMessage(payload, event) : null;
      const mid = message ? getMid(message) : null;
      if (!mid) return;

      const result = await proxyMaxApi(botId, {
        method: "POST",
        path: `/messages?chat_id=${chat.chat_id}`,
        body: { link: { type: "forward", mid } },
      });

      if (result.status >= 200 && result.status < 300) {
        toast.success(t("chats.contextMenu.forwarded"));
        queryClient.invalidateQueries({ queryKey: ["chat-events", botId, chat.chat_id] });
      } else {
        toast.error(t("errors.somethingWentWrong"));
      }
    },
    [botId, chat.chat_id, queryClient, t],
  );

  const handleDelete = useCallback(
    async (event: BotEvent) => {
      const payload = getPayload(event);
      const message = payload ? getMessage(payload, event) : null;
      const mid = message ? getMid(message) : null;
      if (!mid) return;

      const ok = await confirm({
        description: t("chats.contextMenu.confirmDelete"),
        destructive: true,
      });
      if (!ok) return;

      const result = await proxyMaxApi(botId, {
        method: "DELETE",
        path: `/messages?message_id=${mid}`,
      });

      if (result.status >= 200 && result.status < 300) {
        toast.success(t("chats.contextMenu.deleted"));
        queryClient.invalidateQueries({ queryKey: ["chat-events", botId, chat.chat_id] });
      } else {
        const errData = result.data as Record<string, unknown> | null;
        const msg = typeof errData?.message === "string" ? errData.message : t("errors.somethingWentWrong");
        toast.error(msg);
      }
    },
    [botId, chat.chat_id, confirm, queryClient, t],
  );

  const syncHistory = useSyncChatHistory(botId, chat.chat_id);

  const handleSyncHistory = useCallback(() => {
    syncHistory.mutate(undefined, {
      onSuccess: (data) => {
        if (data.synced > 0) {
          toast.success(t("chats.historySynced", { count: data.synced }));
        } else {
          toast.info(t("chats.historyUpToDate", "History is up to date"));
        }
      },
      onError: () => {
        toast.error(t("errors.somethingWentWrong"), { duration: 10000 });
      },
    });
  }, [syncHistory, t]);

  const handleSent = useCallback(() => {
    // Refetch latest messages after sending
    queryClient.invalidateQueries({ queryKey: ["chat-events", botId, chat.chat_id] });
  }, [queryClient, botId, chat.chat_id]);

  return (
    <div className="flex flex-col flex-1 min-h-0">
      <div className="flex items-center gap-2 mb-3 shrink-0">
        {onBack && (
          <Button variant="ghost" size="icon-sm" onClick={onBack} className="min-w-[44px] min-h-[44px]" aria-label={t("common.back", "Back")}>
            <ArrowLeft className="size-4" />
          </Button>
        )}
        <div className="min-w-0 flex-1">
          <h3 className="text-sm font-medium truncate">
            {chat.title ?? `#${chat.chat_id}`}
          </h3>
          {chat.title && (
            <p className="text-[10px] text-muted-foreground">
              ID: {chat.chat_id}
              {chat.chat_type && ` · ${chat.chat_type}`}
              {chat.participants &&
                ` · ${chat.participants} ${t("orgs.members").toLowerCase()}`}
            </p>
          )}
        </div>
        <Button
          variant="ghost"
          size="xs"
          className="shrink-0 text-muted-foreground text-[10px] gap-1"
          onClick={handleSyncHistory}
          disabled={syncHistory.isPending}
        >
          {syncHistory.isPending ? (
            <Loader2 className="size-3 animate-spin" />
          ) : (
            <Download className="size-3" />
          )}
          {t("chats.syncHistory", "Sync")}
        </Button>
        <Button
          variant="ghost"
          size="xs"
          className="shrink-0 text-muted-foreground text-[10px]"
          onClick={() => setShowChatInfo(true)}
        >
          JSON
        </Button>
      </div>

      {/* Chat info JSON modal */}
      <Dialog open={showChatInfo} onOpenChange={setShowChatInfo}>
        <DialogContent className="sm:max-w-lg max-h-[85vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{chat.title ?? `#${chat.chat_id}`}</DialogTitle>
            <DialogDescription className="sr-only">
              {t("chats.chatInfoDescription", "Raw JSON data for this chat")}
            </DialogDescription>
          </DialogHeader>
          <JsonViewer data={chat} maxHeight="60vh" />
        </DialogContent>
      </Dialog>

      {isLoading ? (
        <div className="space-y-3">
          {[1, 2, 3].map((i) => (
            <Skeleton key={i} className="h-16 rounded-lg" />
          ))}
        </div>
      ) : allEvents.length === 0 ? (
        <p className="text-sm text-muted-foreground text-center py-8">
          {t("common.noData")}
        </p>
      ) : (
        <div className="flex flex-col-reverse overflow-y-auto flex-1 min-h-0 max-h-[calc(100dvh-16rem)] sm:max-h-[calc(100dvh-14rem)]">
          {allEvents.map((event, i) => (
            <div key={event.id} className="py-0.5">
              {i === events.length && historyEvents.length > 0 && (
                <div className="flex items-center gap-2 py-2 px-4">
                  <div className="flex-1 border-t border-border" />
                  <span className="text-[10px] text-muted-foreground whitespace-nowrap">
                    {t("chats.olderMessages", "Earlier messages")}
                  </span>
                  <div className="flex-1 border-t border-border" />
                </div>
              )}
              <EventRow
                event={event}
                showSender={showSenderFlags[i]}
                onInspect={setInspectEvent}
                onReply={handleReply}
                onForward={handleForward}
                onDelete={handleDelete}
              />
            </div>
          ))}

          <div ref={sentinelRef} className="shrink-0 py-1">
            {(isFetchingNextPage || isFetchingHistory) && (
              <Loader2 className="size-4 mx-auto animate-spin text-muted-foreground" />
            )}
          </div>
        </div>
      )}

      {/* Message composer */}
      <MessageComposer
        botId={botId}
        chatId={chat.chat_id}
        replyTo={replyTo}
        onClearReply={() => setReplyTo(null)}
        onSent={handleSent}
      />

      {/* JSON inspector */}
      <Dialog
        open={!!inspectEvent}
        onOpenChange={(open) => !open && setInspectEvent(null)}
      >
        <DialogContent className="sm:max-w-2xl max-h-[85vh] overflow-y-auto">
          {inspectEvent && (
            <>
              <DialogHeader>
                <DialogTitle className="flex items-center gap-2">
                  <Badge className="font-mono text-xs">
                    {inspectEvent.update_type}
                  </Badge>
                  <span className="text-xs text-muted-foreground font-normal">
                    {formatDate(inspectEvent.created_at)}
                  </span>
                </DialogTitle>
                <DialogDescription className="sr-only">
                  {t("chats.eventInspectorDescription", "Raw JSON payload for this event")}
                </DialogDescription>
              </DialogHeader>
              <JsonViewer data={inspectEvent.raw_payload} maxHeight="60vh" />
            </>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
