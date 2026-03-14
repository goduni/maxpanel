import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { motion } from "motion/react";
import { extractApiError } from "@/lib/errors";
import { Check, Copy, Key, Loader2, Plus, Trash2 } from "lucide-react";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { formatDate } from "@/lib/utils";
import type { ApiKeyCreateResponse } from "@/lib/api-types";
import {
  useBot,
  useUpdateBot,
  useDeleteBot,
  useApiKeys,
  useCreateApiKey,
  useDeleteApiKey,
} from "../hooks/use-bots";

export function BotSettingsPage() {
  const { t } = useTranslation();
  const confirm = useConfirm();
  const { orgSlug, projectSlug, botId } = useParams<{
    orgSlug: string;
    projectSlug: string;
    botId: string;
  }>();
  const { data: bot, isLoading } = useBot(orgSlug!, projectSlug!, botId!);
  const updateBot = useUpdateBot(orgSlug!, projectSlug!, botId!);
  const deleteBot = useDeleteBot(orgSlug!, projectSlug!, botId!);
  const [name, setName] = useState("");
  const [historyLimit, setHistoryLimit] = useState(0);
  const [error, setError] = useState<string | null>(null);

  // Sync local state when server data arrives
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    if (bot?.name) setName(bot.name);
    // eslint-disable-next-line react-hooks/set-state-in-effect
    if (bot) setHistoryLimit(bot.history_limit);
  }, [bot?.name, bot?.history_limit, bot]);

  if (isLoading) {
    return <Skeleton className="h-32" />;
  }

  if (!bot) return null;

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.15 }}
      className="space-y-6"
    >
      {/* Name */}
      <div className="space-y-2">
        <Label htmlFor="bot-name" className="text-sm font-medium">
          {t("bots.name")}
        </Label>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            setError(null);
            updateBot.mutate(
              { name },
              {
                onSuccess: () => toast.success(t("common.save")),
                onError: (err) => {
                  setError(extractApiError(err, t("errors.somethingWentWrong")));
                },
              },
            );
          }}
          className="flex gap-2"
        >
          <Input
            id="bot-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
            className="max-w-sm"
          />
          <Button type="submit" size="sm" disabled={updateBot.isPending}>
            {t("common.save")}
          </Button>
        </form>
        {error && <p className="text-sm text-destructive">{error}</p>}
      </div>

      <Separator />

      {/* History Limit */}
      <div className="space-y-3">
        <div>
          <h3 className="text-sm font-medium">{t("bots.historyLimit", "Message History Sync")}</h3>
          <p className="text-xs text-muted-foreground mt-1">
            {t("bots.historyLimitDescription", "Number of recent messages to sync per chat from Max API. Set 0 to disable. Older messages are loaded on-the-fly.")}
          </p>
        </div>
        <div className="flex items-center gap-3">
          <Input
            type="number"
            min={0}
            max={10000}
            value={historyLimit}
            onChange={(e) => setHistoryLimit(Math.max(0, Math.min(10000, Number(e.target.value) || 0)))}
            className="w-32"
          />
          <Button
            size="sm"
            disabled={historyLimit === bot?.history_limit || updateBot.isPending}
            onClick={() => {
              updateBot.mutate(
                { history_limit: historyLimit },
                {
                  onSuccess: () => toast.success(t("common.saved", "Saved")),
                  onError: (e) => toast.error(extractApiError(e, t("errors.somethingWentWrong", "Something went wrong"))),
                },
              );
            }}
          >
            {updateBot.isPending && <Loader2 className="size-3 animate-spin mr-1" />}
            {t("common.save", "Save")}
          </Button>
        </div>
      </div>

      <Separator />

      {/* API Keys */}
      <ApiKeysSection botId={botId!} />

      <Separator />

      {/* Delete */}
      <Button
        variant="destructive"
        size="sm"
        className="gap-1.5"
        onClick={async () => {
          const ok = await confirm({
            description: t("common.confirmDelete", { name: bot.name }),
            destructive: true,
          });
          if (!ok) return;
          deleteBot.mutate();
        }}
        disabled={deleteBot.isPending}
      >
        <Trash2 className="size-3.5" />
        {t("bots.delete")}
      </Button>
    </motion.div>
  );
}

function ApiKeysSection({ botId }: { botId: string }) {
  const { t } = useTranslation();
  const confirm = useConfirm();
  const { data: keys, isLoading } = useApiKeys(botId);
  const createKey = useCreateApiKey(botId);
  const deleteKey = useDeleteApiKey(botId);
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [newKeyName, setNewKeyName] = useState("");
  const [createdKey, setCreatedKey] = useState<ApiKeyCreateResponse | null>(
    null,
  );
  const [copied, setCopied] = useState(false);
  const [createError, setCreateError] = useState<string | null>(null);

  const handleCreate = () => {
    setCreateError(null);
    createKey.mutate(
      { name: newKeyName },
      {
        onSuccess: (response) => {
          setCreatedKey(response);
          setNewKeyName("");
        },
        onError: (err) => {
          setCreateError(
            extractApiError(err, t("errors.somethingWentWrong")),
          );
        },
      },
    );
  };

  const handleCopyKey = async () => {
    if (!createdKey) return;
    try {
      await navigator.clipboard.writeText(createdKey.key);
      setCopied(true);
      toast.success(t("common.copied"));
      setTimeout(() => setCopied(false), 2000);
    } catch {
      toast.error(t("errors.somethingWentWrong"));
    }
  };

  const handleCloseCreateDialog = () => {
    setShowCreateDialog(false);
    setCreatedKey(null);
    setNewKeyName("");
    setCreateError(null);
  };

  const handleDelete = async (keyId: string) => {
    const ok = await confirm({
      description: t("bots.apiKeys.deleteConfirm"),
      destructive: true,
    });
    if (!ok) return;
    deleteKey.mutate(keyId, {
      onError: (err) => {
        toast.error(extractApiError(err, t("errors.somethingWentWrong")));
      },
    });
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <Label className="text-sm font-medium flex items-center gap-1.5">
          <Key className="size-4" />
          {t("bots.apiKeys.title")}
        </Label>
        <Button
          variant="outline"
          size="xs"
          className="gap-1.5"
          onClick={() => setShowCreateDialog(true)}
          disabled={keys && keys.length >= 10}
        >
          <Plus className="size-3" />
          {t("bots.apiKeys.create")}
        </Button>
      </div>

      {keys && keys.length >= 10 && (
        <p className="text-xs text-muted-foreground">
          {t("bots.apiKeys.maxKeys")}
        </p>
      )}

      {isLoading ? (
        <Skeleton className="h-16" />
      ) : keys && keys.length > 0 ? (
        <div className="space-y-2">
          {keys.map((key) => (
            <div
              key={key.id}
              className="flex items-center justify-between rounded-lg border border-border/50 p-3"
            >
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2 mb-0.5">
                  <span className="text-sm font-medium">{key.name}</span>
                  <Badge
                    variant={key.is_active ? "default" : "secondary"}
                    className="text-[10px]"
                  >
                    {key.is_active ? t("bots.active") : t("bots.inactive")}
                  </Badge>
                </div>
                <div className="flex items-center gap-3 text-xs text-muted-foreground">
                  <span className="font-mono">{key.key_prefix}...</span>
                  <span>
                    {t("common.createdAt")}: {formatDate(key.created_at)}
                  </span>
                  <span>
                    {t("bots.apiKeys.lastUsed")}:{" "}
                    {key.last_used_at
                      ? formatDate(key.last_used_at)
                      : t("bots.apiKeys.never")}
                  </span>
                </div>
              </div>
              <Button
                variant="ghost"
                size="icon-sm"
                className="text-destructive hover:text-destructive shrink-0"
                onClick={() => handleDelete(key.id)}
                disabled={deleteKey.isPending}
              >
                <Trash2 className="size-3.5" />
              </Button>
            </div>
          ))}
        </div>
      ) : (
        <p className="text-sm text-muted-foreground py-2">
          {t("common.noData")}
        </p>
      )}

      {/* Create / Show Key Dialog */}
      <Dialog open={showCreateDialog} onOpenChange={handleCloseCreateDialog}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>
              {createdKey
                ? t("bots.apiKeys.keyCreated")
                : t("bots.apiKeys.create")}
            </DialogTitle>
            <DialogDescription>
              {createdKey
                ? t("bots.apiKeys.copyWarning")
                : t("bots.apiKeys.title")}
            </DialogDescription>
          </DialogHeader>

          {createdKey ? (
            <div className="space-y-3">
              <div className="flex items-center gap-2">
                <Input
                  readOnly
                  value={createdKey.key}
                  className="font-mono text-xs"
                />
                <Button
                  variant="outline"
                  size="icon"
                  onClick={handleCopyKey}
                >
                  {copied ? (
                    <Check className="size-4" />
                  ) : (
                    <Copy className="size-4" />
                  )}
                </Button>
              </div>
              <p className="text-xs text-amber-600 dark:text-amber-400">
                {t("bots.apiKeys.copyWarning")}
              </p>
            </div>
          ) : (
            <form
              onSubmit={(e) => {
                e.preventDefault();
                handleCreate();
              }}
              className="space-y-3"
            >
              <div className="space-y-1.5">
                <Label htmlFor="api-key-name">{t("bots.apiKeys.name")}</Label>
                <Input
                  id="api-key-name"
                  value={newKeyName}
                  onChange={(e) => setNewKeyName(e.target.value)}
                  placeholder="production"
                  required
                  autoFocus
                />
              </div>
              {createError && (
                <p className="text-sm text-destructive">{createError}</p>
              )}
              <DialogFooter>
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleCloseCreateDialog}
                >
                  {t("common.cancel")}
                </Button>
                <Button
                  type="submit"
                  disabled={createKey.isPending || !newKeyName.trim()}
                >
                  {createKey.isPending && (
                    <Loader2 className="size-3.5 mr-1.5 animate-spin" />
                  )}
                  {t("common.create")}
                </Button>
              </DialogFooter>
            </form>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
