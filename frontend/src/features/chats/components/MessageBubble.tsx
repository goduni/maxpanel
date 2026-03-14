import { memo } from "react";
import { Reply, Forward, Trash2, Code } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { cn } from "@/lib/utils";
import { getInitials } from "@/lib/user-utils";
import type { BotEvent } from "@/lib/api-types";
import {
  getPayload,
  getMessage,
  getBody,
  getSender,
  getAttachments,
  getMarkup,
  isOutboundEvent,
} from "@/features/chats/lib/payload";
import { RichText } from "./RichText";
import { LinkedMessageBlock } from "./LinkedMessageBlock";
import { AttachmentChip } from "./AttachmentChip";

function formatTimestamp(timestamp: number): string {
  return new Date(timestamp).toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  });
}

export const MessageBubble = memo(function MessageBubble({
  event,
  showSender = true,
  onInspect,
  onReply,
  onForward,
  onDelete,
}: {
  event: BotEvent;
  showSender?: boolean;
  onInspect: () => void;
  onReply?: () => void;
  onForward?: () => void;
  onDelete?: () => void;
}) {
  const { t } = useTranslation();
  const payload = getPayload(event);
  const message = payload ? getMessage(payload, event) : null;
  const body = message ? getBody(message) : null;
  const sender = message ? getSender(message, event) : null;
  const text = body && typeof body.text === "string" ? body.text : null;
  const markup = body ? getMarkup(body) : undefined;
  const attachments = body ? getAttachments(body) : [];
  const isEdited = event.update_type === "message_edited";
  const isRemoved = event.update_type === "message_removed";

  // Reply / forward
  const link = message?.link as Record<string, unknown> | undefined;
  const linkType = link && typeof link.type === "string" ? link.type : null;
  const linkMessage = link?.message as Record<string, unknown> | undefined;

  const initials = sender ? getInitials(sender.name) : "?";
  const time = formatTimestamp(event.timestamp);
  const isOutgoing = isOutboundEvent(event) || (sender?.isBot ?? false);

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        <div
          className={cn(
            "flex group",
            isOutgoing ? "justify-end" : "justify-start",
            showSender ? "pt-1" : "pt-px",
          )}
        >
          <div
            className={cn(
              "flex gap-2 max-w-[85%] sm:max-w-[75%]",
              isOutgoing ? "flex-row-reverse" : "flex-row",
            )}
          >
            {/* Avatar */}
            {!isOutgoing && (
              <Avatar
                className={cn(
                  "size-7 mt-auto shrink-0",
                  !showSender && "invisible",
                )}
              >
                <AvatarFallback className="text-[9px] bg-primary/10 text-primary">
                  {initials}
                </AvatarFallback>
              </Avatar>
            )}

            {/* Bubble */}
            <div
              className={cn(
                "rounded-2xl px-3 py-2 min-w-0 relative",
                isOutgoing
                  ? "bg-primary text-primary-foreground rounded-br-sm"
                  : "bg-muted rounded-bl-sm",
              )}
            >
              {/* Sender name */}
              {!isOutgoing && showSender && (
                <p className="text-xs font-semibold text-primary mb-0.5">
                  {sender?.name ?? `User ${event.sender_id}`}
                </p>
              )}

              {/* Reply */}
              {linkType === "reply" && linkMessage && (
                <div
                  className={cn(
                    "border-l-2 pl-2 mb-1.5 rounded-sm",
                    isOutgoing
                      ? "border-primary-foreground/40 bg-primary-foreground/10"
                      : "border-primary/40 bg-primary/5",
                  )}
                >
                  <LinkedMessageBlock
                    type="reply"
                    sender={link?.sender as Record<string, unknown> | undefined}
                    message={linkMessage}
                  />
                </div>
              )}

              {/* Forward */}
              {linkType === "forward" && (
                <div
                  className={cn(
                    "border-l-2 pl-2 mb-1.5 rounded-sm",
                    isOutgoing
                      ? "border-primary-foreground/30 bg-primary-foreground/10"
                      : "border-muted-foreground/30 bg-muted-foreground/5",
                  )}
                >
                  <LinkedMessageBlock
                    type="forward"
                    sender={link?.sender as Record<string, unknown> | undefined}
                    message={linkMessage}
                    label={t("chats.forwarded")}
                    fromChatId={
                      typeof link?.chat_id === "number" ? link.chat_id : undefined
                    }
                  />
                </div>
              )}

              {/* Removed */}
              {isRemoved && (
                <p
                  className={cn(
                    "text-sm italic",
                    isOutgoing
                      ? "text-primary-foreground/70"
                      : "text-muted-foreground",
                  )}
                >
                  {t("chats.messageRemoved")}
                </p>
              )}

              {/* Text */}
              {text && !isRemoved && (
                <p className="text-sm whitespace-pre-wrap break-words">
                  <RichText text={text} markup={markup} />
                </p>
              )}

              {/* Attachments */}
              {attachments.length > 0 && !isRemoved && (
                <div className="flex flex-wrap gap-1.5 mt-1.5">
                  {attachments.map((att, i) => (
                    <AttachmentChip key={i} attachment={att} />
                  ))}
                </div>
              )}

              {/* Edited + time */}
              <div className="flex items-center gap-1 mt-1 select-none justify-end">
                {isEdited && (
                  <span
                    className={cn(
                      "text-[10px] italic",
                      isOutgoing
                        ? "text-primary-foreground/50"
                        : "text-muted-foreground/60",
                    )}
                  >
                    {t("chats.edited")}
                  </span>
                )}
                <span
                  className={cn(
                    "text-[10px]",
                    isOutgoing
                      ? "text-primary-foreground/50"
                      : "text-muted-foreground/60",
                  )}
                >
                  {time}
                </span>
              </div>
            </div>
          </div>
        </div>
      </ContextMenuTrigger>

      <ContextMenuContent className="w-48">
        {onReply && (
          <ContextMenuItem onClick={onReply} className="gap-2">
            <Reply className="size-4" />
            {t("chats.compose.reply")}
          </ContextMenuItem>
        )}
        {onForward && (
          <ContextMenuItem onClick={onForward} className="gap-2">
            <Forward className="size-4" />
            {t("chats.contextMenu.forward")}
          </ContextMenuItem>
        )}
        <ContextMenuItem onClick={onInspect} className="gap-2">
          <Code className="size-4" />
          {t("common.inspectJson")}
        </ContextMenuItem>
        {onDelete && (
          <>
            <ContextMenuSeparator />
            <ContextMenuItem
              onClick={onDelete}
              className="gap-2 text-destructive focus:text-destructive"
            >
              <Trash2 className="size-4" />
              {t("chats.contextMenu.delete")}
            </ContextMenuItem>
          </>
        )}
      </ContextMenuContent>
    </ContextMenu>
  );
});
