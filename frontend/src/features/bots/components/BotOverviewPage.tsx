import { useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { motion } from "motion/react";
import { AlertTriangle, Clock, ExternalLink, Hash, RefreshCw, Wifi } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { formatDate, cn } from "@/lib/utils";
import { useBot, useVerifyBot } from "../hooks/use-bots";
import { JsonViewer } from "@/components/ui/json-viewer";

export function BotOverviewPage() {
  const { t } = useTranslation();
  const { orgSlug, projectSlug, botId } = useParams<{
    orgSlug: string;
    projectSlug: string;
    botId: string;
  }>();
  const { data: bot, isLoading, isError, refetch } = useBot(orgSlug!, projectSlug!, botId!);
  const verifyBot = useVerifyBot(orgSlug!, projectSlug!, botId!);

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-20" />
        <Skeleton className="h-40" />
      </div>
    );
  }

  if (isError) {
    return (
      <div className="text-center py-12 text-destructive">
        <AlertTriangle className="size-8 mx-auto mb-3 text-destructive/60" />
        {t("errors.somethingWentWrong")}
        <Button variant="outline" onClick={() => refetch()} className="ml-2">
          {t("common.retry")}
        </Button>
      </div>
    );
  }

  if (!bot) return null;

  const maxInfo = bot.max_bot_info;
  const info = maxInfo && typeof maxInfo === "object" ? (maxInfo as Record<string, unknown>) : null;
  const botDescription = info && typeof info.description === "string" ? info.description : null;
  const botUsername = info && typeof info.username === "string" ? info.username : null;

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.15 }}
      className="space-y-6"
    >
      {/* Description + link to Max */}
      {(botDescription || botUsername) && (
        <div className="flex items-start justify-between gap-4">
          {botDescription && (
            <p className="text-sm text-muted-foreground">{botDescription}</p>
          )}
          {botUsername && (
            <a
              href={`https://max.ru/${botUsername}`}
              target="_blank"
              rel="noopener noreferrer"
              className="shrink-0 inline-flex items-center gap-1.5 text-xs text-primary hover:underline"
            >
              max.ru/{botUsername}
              <ExternalLink className="size-3" />
            </a>
          )}
        </div>
      )}

      {/* Info grid */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <InfoTile
          icon={Wifi}
          label={t("bots.eventMode")}
          value={t("bots." + bot.event_mode)}
          accent={bot.event_mode === "webhook" ? "emerald" : "blue"}
        />
        <InfoTile
          icon={Hash}
          label={t("bots.overviewLabels.id")}
          value={bot.id.slice(0, 8)}
          mono
        />
        <InfoTile
          icon={Clock}
          label={t("common.createdAt")}
          value={formatDate(bot.created_at)}
        />

      </div>

      {/* Actions */}
      <div className="flex gap-2">
        <Button
          size="sm"
          variant="outline"
          className="gap-1.5"
          onClick={() => {
            verifyBot.mutate(undefined, {
              onSuccess: () => toast.success(t("bots.verified", "Bot verified")),
            });
          }}
          disabled={verifyBot.isPending}
        >
          <RefreshCw
            className={cn("size-3.5", verifyBot.isPending && "animate-spin")}
          />
          {t("bots.verify")}
        </Button>
      </div>

      {/* Max Bot Info */}
      {maxInfo && (
        <>
          <Separator />
          <div className="space-y-2">
            <h3 className="text-sm font-medium">{t("bots.overviewLabels.maxBotInfo")}</h3>
            <JsonViewer data={maxInfo} maxHeight="300px" />
          </div>
        </>
      )}
    </motion.div>
  );
}

function InfoTile({
  icon: Icon,
  label,
  value,
  mono,
  accent,
}: {
  icon: React.ElementType;
  label: string;
  value: string;
  mono?: boolean;
  accent?: "emerald" | "blue";
}) {
  const accentColor = accent === "emerald"
    ? "text-emerald-600 dark:text-emerald-400"
    : accent === "blue"
      ? "text-blue-600 dark:text-blue-400"
      : "";

  return (
    <div className="rounded-lg border border-border/50 p-3 space-y-1.5">
      <div className="flex items-center gap-1.5 text-muted-foreground">
        <Icon className="size-3.5" />
        <span className="text-xs">{label}</span>
      </div>
      <p
        className={cn(
          "text-sm font-medium truncate",
          mono && "font-mono text-xs",
          accentColor,
        )}
      >
        {value}
      </p>
    </div>
  );
}
