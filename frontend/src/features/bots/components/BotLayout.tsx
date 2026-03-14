import { Outlet, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Bot as BotIcon, Power } from "lucide-react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { StatusBadge } from "@/components/ui/status-badge";
import { getMaxBotName } from "@/lib/bot-utils";
import { useBot, useStartBot, useStopBot } from "../hooks/use-bots";

export function BotLayout() {
  const { t } = useTranslation();
  const { orgSlug, projectSlug, botId } = useParams<{
    orgSlug: string;
    projectSlug: string;
    botId: string;
  }>();
  const { data: bot, isLoading } = useBot(orgSlug!, projectSlug!, botId!);
  const startBot = useStartBot(orgSlug!, projectSlug!, botId!);
  const stopBot = useStopBot(orgSlug!, projectSlug!, botId!);

  const maxName = bot ? getMaxBotName(bot.max_bot_info) : null;

  return (
    <div className="flex flex-col gap-6 flex-1 min-h-0">
      {/* Bot header — hero-style */}
      {isLoading ? (
        <div className="flex items-center gap-4">
          <Skeleton className="size-12 rounded-xl" />
          <div className="space-y-2">
            <Skeleton className="h-5 w-40" />
            <Skeleton className="h-4 w-24" />
          </div>
        </div>
      ) : bot ? (
        <div className="flex items-center gap-4">
          <div className="size-12 rounded-xl bg-primary/10 flex items-center justify-center shrink-0">
            <BotIcon className="size-6 text-primary" />
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <h1 className="text-xl font-semibold tracking-tight truncate">
                {bot.name}
              </h1>
              <StatusBadge active={bot.is_active} />
            </div>
            <div className="flex items-center gap-2 mt-0.5">
              {maxName && (
                <span className="text-sm text-muted-foreground truncate">
                  {maxName}
                </span>
              )}
              <Badge variant="outline" className="text-[10px] px-1.5 py-0">
                {t("bots." + bot.event_mode)}
              </Badge>
            </div>
          </div>

          {/* Quick action */}
          <Button
            size="sm"
            variant={bot.is_active ? "outline" : "default"}
            className="gap-1.5 shrink-0"
            onClick={() => {
              if (bot.is_active) {
                stopBot.mutate(undefined, {
                  onSuccess: () => toast.success(t("bots.stop")),
                });
              } else {
                startBot.mutate(undefined, {
                  onSuccess: () => toast.success(t("bots.start")),
                });
              }
            }}
            disabled={startBot.isPending || stopBot.isPending}
          >
            <Power className="size-3.5" />
            {bot.is_active ? t("bots.stop") : t("bots.start")}
          </Button>
        </div>
      ) : null}

      {/* Content — no duplicate tab nav, sidebar handles navigation */}
      <div className="flex flex-col flex-1 min-h-0">
        <Outlet />
      </div>
    </div>
  );
}
