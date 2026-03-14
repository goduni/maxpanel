import { memo, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Hash, Loader2, Megaphone, MessageSquare, Search, Users } from "lucide-react";
import { Input } from "@/components/ui/input";
import { EmptyState } from "@/components/layout/EmptyState";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { cn, formatDate } from "@/lib/utils";
import { useChats } from "@/features/events/hooks/use-events";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import type { BotChat } from "@/lib/api-types";

const CHAT_TYPE_ICONS: Record<string, React.ElementType> = {
  dialog: MessageSquare,
  chat: Hash,
  channel: Megaphone,
};

const CHAT_TYPE_COLORS: Record<string, string> = {
  dialog: "text-blue-600 dark:text-blue-400",
  chat: "text-emerald-600 dark:text-emerald-400",
  channel: "text-amber-600 dark:text-amber-400",
};

export function ChatList({
  botId,
  selectedChatId,
  onSelect,
}: {
  botId: string;
  selectedChatId: number | null;
  onSelect: (chat: BotChat) => void;
}) {
  const { t } = useTranslation();
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const { data, isLoading, fetchNextPage, hasNextPage, isFetchingNextPage } =
    useChats(botId, debouncedSearch || undefined);

  // Debounce search input to avoid excessive API calls
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(null);
  const handleSearch = (value: string) => {
    setSearch(value);
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => setDebouncedSearch(value.trim()), 300);
  };

  const chats = useMemo(
    () => data?.pages.flatMap((p) => p.data) ?? [],
    [data],
  );

  const sentinelRef = useInfiniteScroll({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
  });

  if (isLoading) {
    return (
      <div className="space-y-2">
        {[1, 2, 3].map((i) => (
          <Skeleton key={i} className="h-16 rounded-lg" />
        ))}
      </div>
    );
  }

  if (chats.length === 0) {
    return <EmptyState icon={MessageSquare} message={t("chats.emptyState")} />;
  }

  return (
    <div className="space-y-2" role="listbox">
      <div className="relative">
        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground pointer-events-none" />
        <Input
          value={search}
          onChange={(e) => handleSearch(e.target.value)}
          placeholder={t("chats.search", "Search chats...")}
          className="h-10 sm:h-8 pl-8 text-sm sm:text-xs"
          inputMode="search"
          autoCapitalize="off"
        />
      </div>
      {chats.map((chat) => (
        <ChatListItem
          key={chat.chat_id}
          chat={chat}
          selected={selectedChatId === chat.chat_id}
          onSelect={() => onSelect(chat)}
        />
      ))}
      <div ref={sentinelRef} className="py-2">
        {isFetchingNextPage && (
          <Loader2 className="size-4 mx-auto animate-spin text-muted-foreground" />
        )}
      </div>
    </div>
  );
}

const ChatListItem = memo(function ChatListItem({
  chat,
  selected,
  onSelect,
}: {
  chat: BotChat;
  selected: boolean;
  onSelect: () => void;
}) {
  const TypeIcon = chat.chat_type
    ? (CHAT_TYPE_ICONS[chat.chat_type] ?? MessageSquare)
    : MessageSquare;
  const typeColor = chat.chat_type
    ? (CHAT_TYPE_COLORS[chat.chat_type] ?? "")
    : "";

  return (
    <button
      role="option"
      aria-selected={selected}
      onClick={onSelect}
      className={cn(
        "w-full text-left rounded-lg px-3 py-2.5 transition-all focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none",
        selected
          ? "bg-muted border border-border"
          : "hover:bg-muted/50 border border-transparent",
      )}
    >
      <div className="flex items-center gap-3">
        {chat.icon_url ? (
          <Avatar className="size-9 shrink-0">
            <AvatarImage src={chat.icon_url} loading="lazy" />
            <AvatarFallback>
              <TypeIcon className={cn("size-4", typeColor)} />
            </AvatarFallback>
          </Avatar>
        ) : (
          <div className="size-9 rounded-full bg-muted flex items-center justify-center shrink-0">
            <TypeIcon className={cn("size-4", typeColor)} />
          </div>
        )}
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-1.5">
            <span className="text-sm font-medium truncate">
              {chat.title ?? `#${chat.chat_id}`}
            </span>
            {chat.chat_type && (
              <Badge
                variant="outline"
                className="text-[9px] px-1 py-0 shrink-0"
              >
                {chat.chat_type}
              </Badge>
            )}
          </div>
          <div className="flex items-center gap-2 mt-0.5">
            {chat.participants && (
              <span className="text-[10px] text-muted-foreground flex items-center gap-0.5">
                <Users className="size-3" />
                {chat.participants}
              </span>
            )}
            {chat.last_event_at && (
              <span className="text-[10px] text-muted-foreground truncate">
                {formatDate(chat.last_event_at)}
              </span>
            )}
          </div>
        </div>
      </div>
    </button>
  );
});
