import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

export function StatusBadge({ active }: { active: boolean }) {
  const { t } = useTranslation();

  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 text-[10px] font-medium px-2 py-0.5 rounded-full",
        active
          ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
          : "bg-muted text-muted-foreground",
      )}
    >
      <span
        className={cn(
          "size-1.5 rounded-full",
          active
            ? "bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.6)] animate-pulse"
            : "bg-muted-foreground/40",
        )}
      />
      {active ? t("bots.active") : t("bots.inactive")}
    </span>
  );
}
