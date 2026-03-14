import { cn } from "@/lib/utils";
import { getSenderDirect, getAttachments, getMarkup } from "@/features/chats/lib/payload";
import { RichText } from "./RichText";
import { AttachmentChip } from "./AttachmentChip";

export function LinkedMessageBlock({
  type,
  sender,
  message,
  label,
  fromChatId,
}: {
  type: "reply" | "forward";
  sender?: Record<string, unknown>;
  message?: Record<string, unknown>;
  label?: string;
  fromChatId?: number;
}) {
  const senderName = sender
    ? getSenderDirect(sender)?.name
    : null;
  const text =
    message && typeof message.text === "string" ? message.text : null;
  const msgMarkup = message ? getMarkup(message) : undefined;
  const attachments = message ? getAttachments(message) : [];

  const isReply = type === "reply";

  return (
    <div className="mt-1 mb-1 space-y-1">
      {/* Header: label + sender + source chat */}
      <div className="flex items-center gap-1 flex-wrap">
        {label && (
          <span className="text-[10px] text-muted-foreground italic">
            {label}
          </span>
        )}
        {senderName && (
          <span className="text-[10px] text-primary font-medium">
            {senderName}
          </span>
        )}
        {fromChatId != null && (
          <span className="text-[10px] text-muted-foreground">
            · #{fromChatId}
          </span>
        )}
      </div>

      {/* Full text with markup */}
      {text && (
        <p
          className={cn(
            "text-xs",
            isReply
              ? "text-muted-foreground line-clamp-2"
              : "whitespace-pre-wrap break-words",
          )}
        >
          <RichText text={text} markup={msgMarkup} />
        </p>
      )}

      {/* Attachments -- full render */}
      {attachments.length > 0 && (
        <div className="flex flex-wrap gap-1.5 mt-0.5">
          {attachments.map((att, i) => (
            <AttachmentChip key={i} attachment={att} />
          ))}
        </div>
      )}
    </div>
  );
}
