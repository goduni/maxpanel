import { useState } from "react";
import { useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { RefreshCw } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useSyncChats } from "@/features/events/hooks/use-events";
import { useIsMobile } from "@/hooks/use-media-query";
import type { BotChat } from "@/lib/api-types";
import { ChatList } from "./ChatList";
import { ChatThread } from "./ChatThread";

export function ChatsPage() {
  const { t } = useTranslation();
  const { botId } = useParams<{ botId: string }>();
  const isMobile = useIsMobile();
  const syncChats = useSyncChats(botId!);
  const [selectedChat, setSelectedChat] = useState<BotChat | null>(null);

  const handleSync = () => {
    syncChats.mutate(undefined, {
      onSuccess: (data) => {
        toast.success(`${t("chats.synced")}: ${data.synced}`);
      },
      onError: () => {
        toast.error(t("errors.somethingWentWrong"));
      },
    });
  };

  if (isMobile && selectedChat) {
    // h-[calc(100dvh - 3.5rem header - 4rem tabs - 2rem padding)]
    return (
      <div className="flex flex-col -m-4 p-4 overflow-hidden" style={{ height: "calc(100dvh - 7.5rem)" }}>
        <ChatThread
          botId={botId!}
          chat={selectedChat}
          onBack={() => setSelectedChat(null)}
        />
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-3 flex-1 min-h-0">
      <div className="flex justify-end">
        <Button
          variant="outline"
          size="sm"
          className="gap-1.5"
          onClick={handleSync}
          disabled={syncChats.isPending}
        >
          <RefreshCw
            className={cn("size-3.5", syncChats.isPending && "animate-spin")}
          />
          {t("chats.sync")}
        </Button>
      </div>

      <div className="flex gap-4 flex-1 min-h-0">
        <div
          className={
            isMobile
              ? "w-full"
              : "w-80 shrink-0 border-r border-border pr-4 overflow-y-auto"
          }
        >
          <ChatList
            botId={botId!}
            selectedChatId={selectedChat?.chat_id ?? null}
            onSelect={setSelectedChat}
          />
        </div>

        {!isMobile && (
          <div className="flex-1 min-w-0 flex flex-col">
            {selectedChat ? (
              <ChatThread botId={botId!} chat={selectedChat} />
            ) : (
              <div className="flex items-center justify-center flex-1 text-muted-foreground text-sm">
                {t("common.selectChat")}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
