import { memo } from "react";
import { useTranslation } from "react-i18next";
import type { BotEvent } from "@/lib/api-types";

export const SystemEvent = memo(function SystemEvent({
  event,
  onInspect,
}: {
  event: BotEvent;
  onInspect: () => void;
}) {
  const { t } = useTranslation();
  const label = t(`chats.systemEvents.${event.update_type}`, {
    defaultValue: t("chats.systemEvents.unknown"),
  });
  const time = new Date(event.timestamp).toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  });

  return (
    <div className="flex justify-center py-2">
      <button
        onClick={onInspect}
        aria-label={t("common.inspectJson")}
        className="inline-flex items-center gap-1.5 text-[11px] text-muted-foreground bg-muted/60 hover:bg-muted rounded-full px-3 py-1 transition-colors"
      >
        <span>{label}</span>
        <span className="text-muted-foreground/50">·</span>
        <span className="text-muted-foreground/50">{time}</span>
      </button>
    </div>
  );
});
