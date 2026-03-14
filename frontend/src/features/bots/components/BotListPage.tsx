import { useState } from "react";
import { Link, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { motion } from "motion/react";
import { Bot as BotIcon, Plus, Power } from "lucide-react";
import { EmptyState } from "@/components/layout/EmptyState";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { PageHeader } from "@/components/layout/PageHeader";
import { StatusBadge } from "@/components/ui/status-badge";
import { cn } from "@/lib/utils";
import { getMaxBotName } from "@/lib/bot-utils";
import { extractApiError } from "@/lib/errors";
import { useBots, useCreateBot, useStartBot, useStopBot } from "../hooks/use-bots";
import type { Bot } from "@/lib/api-types";

export function BotListPage() {
  const { t } = useTranslation();
  const { orgSlug, projectSlug } = useParams<{
    orgSlug: string;
    projectSlug: string;
  }>();
  const { data, isLoading, isError, refetch } = useBots(orgSlug!, projectSlug!);
  const bots = data?.data ?? [];

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.15 }}
      className="space-y-6"
    >
      <PageHeader
        title={t("bots.title")}
        actions={<CreateBotDialog orgSlug={orgSlug!} projectSlug={projectSlug!} />}
      />

      {isLoading && (
        <div className="grid gap-3 sm:grid-cols-2">
          {[1, 2, 3].map((i) => (
            <Skeleton key={i} className="h-28 rounded-lg" />
          ))}
        </div>
      )}

      {isError && (
        <div className="text-center py-12 text-destructive">
          {t("errors.somethingWentWrong")}
          <Button variant="outline" onClick={() => refetch()} className="ml-2">
            {t("common.retry")}
          </Button>
        </div>
      )}

      {!isLoading && !isError && bots.length === 0 && (
        <EmptyState
          icon={BotIcon}
          message={t("bots.emptyState")}
          actions={<CreateBotDialog orgSlug={orgSlug!} projectSlug={projectSlug!} />}
        />
      )}

      {!isLoading && !isError && bots.length > 0 && (
        <div className="grid gap-3 sm:grid-cols-2">
          {bots.map((bot, i) => (
            <motion.div
              key={bot.id}
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3, delay: Math.min(i * 0.03, 0.3) }}
            >
              <BotCard
                bot={bot}
                orgSlug={orgSlug!}
                projectSlug={projectSlug!}
              />
            </motion.div>
          ))}
        </div>
      )}
    </motion.div>
  );
}

function BotCard({
  bot,
  orgSlug,
  projectSlug,
}: {
  bot: Bot;
  orgSlug: string;
  projectSlug: string;
}) {
  const { t } = useTranslation();
  const startBot = useStartBot(orgSlug, projectSlug, bot.id);
  const stopBot = useStopBot(orgSlug, projectSlug, bot.id);

  const toggleBot = bot.is_active ? stopBot : startBot;

  const maxName = getMaxBotName(bot.max_bot_info);

  return (
    <Card className="p-4 hover:bg-card/80 transition-all duration-200 group">
      <div className="flex items-start gap-3">
        <Link
          to={`/${orgSlug}/${projectSlug}/bots/${bot.id}`}
          className="flex items-start gap-3 min-w-0 flex-1"
        >
          <div className="size-9 rounded-md bg-primary/10 flex items-center justify-center shrink-0 group-hover:bg-primary/15 transition-colors">
            <BotIcon className="size-4 text-primary" />
          </div>
          <div className="min-w-0 flex-1">
            <h3 className="font-medium text-sm truncate">{bot.name}</h3>
            {maxName && (
              <p className="text-xs text-muted-foreground truncate">
                {String(maxName)}
              </p>
            )}
            <div className="flex items-center gap-2 mt-1.5">
              <StatusBadge active={bot.is_active} />
              <Badge variant="outline" className="text-[10px] px-1.5 py-0">
                {t("bots." + bot.event_mode)}
              </Badge>
            </div>
          </div>
        </Link>

        <Button
          variant="ghost"
          size="icon-sm"
          aria-label={bot.is_active ? t("bots.stop") : t("bots.start")}
          onClick={(e) => {
            e.preventDefault();
            toggleBot.mutate();
          }}
          disabled={toggleBot.isPending}
          className={cn(
            "shrink-0",
            bot.is_active
              ? "text-primary hover:text-primary"
              : "text-muted-foreground",
          )}
        >
          <Power className="size-4" />
        </Button>
      </div>
    </Card>
  );
}

function CreateBotDialog({
  orgSlug,
  projectSlug,
}: {
  orgSlug: string;
  projectSlug: string;
}) {
  const { t } = useTranslation();
  const createBot = useCreateBot(orgSlug, projectSlug);
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [accessToken, setAccessToken] = useState("");
  const [eventMode, setEventMode] = useState<"webhook" | "polling">("polling");
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    createBot.mutate(
      { name, access_token: accessToken, event_mode: eventMode },
      {
        onSuccess: () => {
          setOpen(false);
          setName("");
          setAccessToken("");
          setEventMode("polling");
        },
        onError: (err) => {
          setError(extractApiError(err, t("errors.somethingWentWrong")));
        },
      },
    );
  };

  return (
    <Dialog open={open} onOpenChange={(v) => {
      setOpen(v);
      if (!v) {
        setName("");
        setAccessToken(""); // Clear sensitive token on any close
        setEventMode("polling");
        setError(null);
      }
    }}>
      <DialogTrigger asChild>
        <Button size="sm" className="gap-1.5">
          <Plus className="size-3.5" />
          {t("bots.create")}
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("bots.create")}</DialogTitle>
          <DialogDescription className="sr-only">
            {t("bots.create")}
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          {error && (
            <div className="rounded-md bg-destructive/10 border border-destructive/20 px-3 py-2 text-sm text-destructive">
              {error}
            </div>
          )}
          <div className="space-y-2">
            <Label>{t("bots.name")}</Label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
              minLength={1}
              maxLength={255}
              autoFocus
            />
          </div>
          <div className="space-y-2">
            <Label>{t("bots.accessToken")}</Label>
            <Input
              value={accessToken}
              onChange={(e) => setAccessToken(e.target.value)}
              required
              minLength={1}
              maxLength={512}
              type="password"
              className="font-mono"
            />
          </div>
          <div className="space-y-2">
            <Label>{t("bots.eventMode")}</Label>
            <div className="flex gap-2">
              <Button
                type="button"
                variant={eventMode === "polling" ? "default" : "outline"}
                size="sm"
                onClick={() => setEventMode("polling")}
              >
                {t("bots.polling")}
              </Button>
              <Button
                type="button"
                variant={eventMode === "webhook" ? "default" : "outline"}
                size="sm"
                onClick={() => setEventMode("webhook")}
              >
                {t("bots.webhook")}
              </Button>
            </div>
          </div>
          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => setOpen(false)}
            >
              {t("common.cancel")}
            </Button>
            <Button type="submit" disabled={createBot.isPending}>
              {createBot.isPending ? t("common.loading") : t("common.create")}
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
