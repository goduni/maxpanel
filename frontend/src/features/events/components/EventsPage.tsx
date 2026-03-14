import { memo, useMemo, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useQueryClient } from "@tanstack/react-query";
import { useVirtualizer } from "@tanstack/react-virtual";
import {
  ArrowDownLeft,
  ArrowUpRight,
  Clock,
  Filter,
  Hash,
  Loader2,
  Pause,
  Play,
  RefreshCw,
  ScrollText,
  User,
} from "lucide-react";
import { toast } from "sonner";
import { EmptyState } from "@/components/layout/EmptyState";
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
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn, formatDate } from "@/lib/utils";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { useEvents } from "../hooks/use-events";
import type { BotEvent } from "@/lib/api-types";

export function EventsPage() {
  const { t } = useTranslation();

  const AUTO_REFRESH_OPTIONS = useMemo(
    () =>
      [
        { label: t("events.autoRefreshIntervals.5s"), value: 5000 },
        { label: t("events.autoRefreshIntervals.10s"), value: 10000 },
        { label: t("events.autoRefreshIntervals.30s"), value: 30000 },
        { label: t("events.autoRefreshIntervals.1m"), value: 60000 },
      ] as const,
    [t],
  );
  const { botId } = useParams<{ botId: string }>();
  const queryClient = useQueryClient();
  const [autoRefresh, setAutoRefresh] = useState<number | false>(false);
  const [directionFilter, setDirectionFilter] = useState<
    "inbound" | "outbound" | undefined
  >(undefined);
  const {
    data,
    isLoading,
    isError,
    isFetching,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
    refetch,
  } = useEvents(botId!, autoRefresh, directionFilter);

  const [selectedEvent, setSelectedEvent] = useState<BotEvent | null>(null);
  const [typeFilter, setTypeFilter] = useState<string | null>(null);

  const allEvents = useMemo(
    () => data?.pages.flatMap((p) => p.data) ?? [],
    [data?.pages],
  );

  const filteredEvents = useMemo(
    () =>
      typeFilter
        ? allEvents.filter((e) => e.update_type === typeFilter)
        : allEvents,
    [allEvents, typeFilter],
  );

  const eventTypes = useMemo(
    () => [...new Set(allEvents.map((e) => e.update_type))].sort(),
    [allEvents],
  );

  const handleRefresh = () => {
    queryClient.invalidateQueries({ queryKey: ["events", botId, { direction: directionFilter }] });
  };

  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: filteredEvents.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 60,
    overscan: 5,
  });

  // Infinite scroll
  const sentinelRef = useInfiniteScroll({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
  });

  if (isLoading) {
    return (
      <div className="max-w-3xl mx-auto space-y-3">
        {[1, 2, 3, 4, 5].map((i) => (
          <Skeleton key={i} className="h-16 rounded-lg" />
        ))}
      </div>
    );
  }

  if (isError) {
    return (
      <div className="text-center py-12 text-destructive">
        {t("errors.somethingWentWrong")}
        <Button variant="outline" onClick={() => refetch()} className="ml-2">
          {t("common.retry")}
        </Button>
      </div>
    );
  }

  if (allEvents.length === 0) {
    const emptyMsg = directionFilter
      ? t("common.noData")
      : t("events.emptyState");
    return <EmptyState icon={ScrollText} message={emptyMsg} />;
  }

  const activeInterval = AUTO_REFRESH_OPTIONS.find(
    (o) => o.value === autoRefresh,
  );

  return (
    <div className="max-w-3xl mx-auto">
      {/* Toolbar */}
      <div className="flex items-center gap-2 mb-4 flex-wrap">
        {/* Refresh */}
        <Button
          variant="outline"
          size="xs"
          className="gap-1.5"
          onClick={handleRefresh}
          disabled={isFetching && !isFetchingNextPage}
        >
          <RefreshCw
            className={cn(
              "size-3",
              isFetching && !isFetchingNextPage && "animate-spin",
            )}
          />
          {t("common.retry")}
        </Button>

        {/* Auto-refresh toggle */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant={autoRefresh ? "default" : "outline"}
              size="xs"
              className="gap-1.5"
            >
              {autoRefresh ? (
                <Pause className="size-3" />
              ) : (
                <Play className="size-3" />
              )}
              {autoRefresh
                ? `${t("events.autoRefresh")}: ${activeInterval?.label}`
                : t("events.autoRefresh")}
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start">
            {autoRefresh && (
              <DropdownMenuItem onClick={() => setAutoRefresh(false)}>
                <Pause className="size-3.5 mr-1.5" />
                {t("bots.stop")}
              </DropdownMenuItem>
            )}
            {AUTO_REFRESH_OPTIONS.map((opt) => (
              <DropdownMenuItem
                key={opt.value}
                onClick={() => setAutoRefresh(opt.value)}
                className={cn(autoRefresh === opt.value && "bg-accent font-medium")}
              >
                {opt.label}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>

        {/* Direction filter */}
        <div role="group" aria-label={t("events.direction")} className="flex items-center gap-0.5 border rounded-md p-0.5">
          <Button
            size="xs"
            variant={directionFilter === undefined ? "default" : "ghost"}
            onClick={() => setDirectionFilter(undefined)}
            aria-pressed={directionFilter === undefined}
            className="text-xs h-6 px-2"
          >
            {t("events.allDirections")}
          </Button>
          <Button
            size="xs"
            variant={directionFilter === "inbound" ? "default" : "ghost"}
            onClick={() => setDirectionFilter("inbound")}
            aria-pressed={directionFilter === "inbound"}
            className="text-xs h-6 px-2 gap-1"
          >
            <ArrowDownLeft className="size-3" />
            {t("events.inbound")}
          </Button>
          <Button
            size="xs"
            variant={directionFilter === "outbound" ? "default" : "ghost"}
            onClick={() => setDirectionFilter("outbound")}
            aria-pressed={directionFilter === "outbound"}
            className="text-xs h-6 px-2 gap-1"
          >
            <ArrowUpRight className="size-3" />
            {t("events.outbound")}
          </Button>
        </div>

        {/* Spacer */}
        <div className="flex-1" />

        {/* Filter */}
        {eventTypes.length > 1 && (
          <div role="group" aria-label={t("events.filterByType")} className="flex items-center gap-1.5 overflow-x-auto pb-1 -mb-1 [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none] [scrollbar-width:none]">
            <Filter className="size-3 text-muted-foreground shrink-0" />
            <Button
              size="xs"
              variant={typeFilter === null ? "default" : "ghost"}
              onClick={() => setTypeFilter(null)}
              aria-pressed={typeFilter === null}
              className="text-xs"
            >
              {t("common.all")}
            </Button>
            {eventTypes.map((type) => (
              <Button
                key={type}
                size="xs"
                variant={typeFilter === type ? "default" : "ghost"}
                onClick={() => setTypeFilter(type)}
                aria-pressed={typeFilter === type}
                className="text-xs font-mono"
              >
                {type}
              </Button>
            ))}
          </div>
        )}
      </div>

      {/* Virtualized event list */}
      <div
        ref={parentRef}
        className="overflow-auto"
        style={{ maxHeight: "calc(100dvh - 200px)" }}
      >
        <div
          style={{
            height: `${virtualizer.getTotalSize()}px`,
            width: "100%",
            position: "relative",
          }}
        >
          {virtualizer.getVirtualItems().map((virtualRow) => {
            const event = filteredEvents[virtualRow.index];
            return (
              <div
                key={event.id}
                data-index={virtualRow.index}
                ref={virtualizer.measureElement}
                style={{
                  position: "absolute",
                  top: 0,
                  left: 0,
                  width: "100%",
                  transform: `translateY(${virtualRow.start}px)`,
                }}
              >
                <div className="pb-2">
                  <EventRow
                    event={event}
                    onClick={() => setSelectedEvent(event)}
                  />
                </div>
              </div>
            );
          })}
        </div>

        {/* Infinite scroll sentinel */}
        <div ref={sentinelRef} className="py-6 text-center">
          {isFetchingNextPage && (
            <Loader2 className="size-5 mx-auto animate-spin text-muted-foreground" />
          )}
        </div>
      </div>

      {/* Event detail dialog */}
      <Dialog
        open={!!selectedEvent}
        onOpenChange={(open) => !open && setSelectedEvent(null)}
      >
        <DialogContent className="sm:max-w-2xl max-h-[85vh] overflow-y-auto">
          <DialogDescription className="sr-only">
            {t("events.payload")}
          </DialogDescription>
          {selectedEvent && <EventDetail event={selectedEvent} />}
        </DialogContent>
      </Dialog>
    </div>
  );
}

const EventRow = memo(function EventRow({
  event,
  onClick,
}: {
  event: BotEvent;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "w-full text-left rounded-lg border border-border/50 p-3",
        "hover:border-border hover:bg-muted/30 transition-all",
        "focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none",
      )}
    >
      <div className="flex items-center justify-between mb-1.5">
        <div className="flex items-center gap-1.5">
          {event.direction && (
            <span
              className={cn(
                "inline-flex items-center text-[10px]",
                event.direction === "outbound"
                  ? "text-blue-500"
                  : "text-green-500",
              )}
              title={event.direction}
            >
              {event.direction === "outbound" ? (
                <ArrowUpRight className="size-3" />
              ) : (
                <ArrowDownLeft className="size-3" />
              )}
            </span>
          )}
          <Badge variant="secondary" className="text-[10px] font-mono">
            {event.update_type}
          </Badge>
          {event.source && (
            <Badge variant="outline" className="text-[9px] font-mono">
              {event.source}
            </Badge>
          )}
        </div>
        <span className="text-[10px] text-muted-foreground flex items-center gap-1">
          <Clock className="size-3" />
          {formatDate(event.created_at)}
        </span>
      </div>
      <div className="flex items-center gap-3 text-xs text-muted-foreground">
        {event.chat_id != null && (
          <span className="flex items-center gap-1">
            <Hash className="size-3" />
            {event.chat_id}
          </span>
        )}
        {event.sender_id != null && (
          <span className="flex items-center gap-1">
            <User className="size-3" />
            {event.sender_id}
          </span>
        )}
        {event.chat_id == null && event.sender_id == null && (
          <span className="text-muted-foreground/40 italic">
            {event.update_type}
          </span>
        )}
      </div>
    </button>
  );
});

function EventDetail({ event }: { event: BotEvent }) {
  const { t } = useTranslation();

  const copyId = () => {
    navigator.clipboard.writeText(event.id);
    toast.success(t("common.copied"));
  };

  return (
    <>
      <DialogHeader>
        <DialogTitle className="flex items-center gap-3 flex-wrap">
          {event.direction && (
            <span
              className={cn(
                "inline-flex items-center",
                event.direction === "outbound"
                  ? "text-blue-500"
                  : "text-green-500",
              )}
            >
              {event.direction === "outbound" ? (
                <ArrowUpRight className="size-4" />
              ) : (
                <ArrowDownLeft className="size-4" />
              )}
            </span>
          )}
          <Badge className="font-mono text-xs px-2.5 py-1">
            {event.update_type}
          </Badge>
          {event.source && (
            <Badge variant="outline" className="font-mono text-xs px-2 py-0.5">
              {t("events.source")}: {event.source}
            </Badge>
          )}
          <span className="text-xs text-muted-foreground font-normal flex items-center gap-1">
            <Clock className="size-3" />
            {formatDate(event.created_at)}
          </span>
        </DialogTitle>
      </DialogHeader>

      {/* ID — full width, copyable */}
      <button
        onClick={copyId}
        className="w-full text-left rounded-lg border border-border/50 hover:border-border px-3 py-2.5 mt-4 transition-colors group"
      >
        <p className="text-[10px] text-muted-foreground uppercase tracking-wider">
          {t("events.detail.id")}
        </p>
        <p className="text-xs font-mono mt-0.5 group-hover:text-primary transition-colors break-all">
          {event.id}
        </p>
      </button>

      {/* Chat / Sender / Update ID — second row */}
      {(event.chat_id != null || event.sender_id != null || event.max_update_id) && (
        <div className="grid grid-cols-2 sm:grid-cols-3 gap-2 mt-2">
          {event.chat_id != null && (
            <div className="rounded-lg border border-border/50 px-3 py-2.5">
              <p className="text-[10px] text-muted-foreground uppercase tracking-wider">
                {t("events.chatId")}
              </p>
              <p className="text-sm font-medium mt-0.5">{event.chat_id}</p>
            </div>
          )}
          {event.sender_id != null && (
            <div className="rounded-lg border border-border/50 px-3 py-2.5">
              <p className="text-[10px] text-muted-foreground uppercase tracking-wider">
                {t("events.senderId")}
              </p>
              <p className="text-sm font-medium mt-0.5">{event.sender_id}</p>
            </div>
          )}
          {event.max_update_id && (
            <div className="rounded-lg border border-border/50 px-3 py-2.5">
              <p className="text-[10px] text-muted-foreground uppercase tracking-wider">
                {t("events.detail.updateId")}
              </p>
              <p className="text-sm font-mono mt-0.5">{event.max_update_id}</p>
            </div>
          )}
        </div>
      )}

      {/* Payload */}
      <div className="mt-4 space-y-2">
        <h3 className="text-sm font-medium">{t("events.payload")}</h3>
        <JsonViewer data={event.raw_payload} maxHeight="50vh" />
      </div>
    </>
  );
}
