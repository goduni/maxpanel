import { useState } from "react";
import { useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { motion } from "motion/react";
import { Clock, Play, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { JsonViewer } from "@/components/ui/json-viewer";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import * as botApi from "@/features/bots/api";
import { cn } from "@/lib/utils";

type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

interface HistoryEntry {
  id: string;
  method: HttpMethod;
  path: string;
  body: string;
  status: number | null;
  duration: number;
  response: string;
  timestamp: number;
}

const METHOD_COLORS: Record<HttpMethod, string> = {
  GET: "text-emerald-600 dark:text-emerald-400",
  POST: "text-blue-600 dark:text-blue-400",
  PUT: "text-amber-600 dark:text-amber-400",
  PATCH: "text-orange-600 dark:text-orange-400",
  DELETE: "text-red-600 dark:text-red-400",
};

const COMMON_PATHS = [
  "/chats",
  "/messages",
  "/me",
  "/subscriptions",
  "/updates",
];

const VALID_METHODS = new Set<string>(["GET", "POST", "PUT", "PATCH", "DELETE"]);
const SAFE_PATH_RE = /^\/[a-zA-Z0-9/_\-?=&+]*$/;

function loadHistory(storageKey: string): HistoryEntry[] {
  try {
    const raw: unknown[] = JSON.parse(
      localStorage.getItem(storageKey) ?? "[]",
    );
    return raw
      .filter((e): e is HistoryEntry => {
        if (typeof e !== "object" || e === null) return false;
        const entry = e as HistoryEntry;
        return (
          VALID_METHODS.has(entry.method) &&
          typeof entry.path === "string" &&
          entry.path.length <= 2048 &&
          typeof entry.timestamp === "number"
        );
      })
      .map((e) => (e.id ? e : { ...e, id: crypto.randomUUID() }));
  } catch {
    return [];
  }
}

export function ApiConsolePage() {
  const { t } = useTranslation();
  const { botId } = useParams<{ botId: string }>();

  const [method, setMethod] = useState<HttpMethod>("GET");
  const [path, setPath] = useState("/me");
  const [body, setBody] = useState("");
  const [sending, setSending] = useState(false);
  const [response, setResponse] = useState<{
    status: number;
    data: unknown;
    duration: number;
  } | null>(null);
  const [error, setError] = useState<string | null>(null);

  // History from localStorage (validated on load)
  const storageKey = `maxpanel-console-${botId}`;
  const [history, setHistory] = useState<HistoryEntry[]>(() =>
    loadHistory(storageKey),
  );

  const saveHistory = (entries: HistoryEntry[]) => {
    // Limit entries and expire old ones (7 days)
    const maxAge = 7 * 24 * 60 * 60 * 1000;
    const trimmed = entries
      .filter((e) => Date.now() - e.timestamp < maxAge)
      .slice(0, 20);
    setHistory(trimmed);
    localStorage.setItem(storageKey, JSON.stringify(trimmed));
  };

  const handleSend = async () => {
    if (!botId || sending) return;

    // Validate path: safe characters only, no traversal, max 2048 chars
    if (!path || path.length > 2048 || path.includes("..") || !SAFE_PATH_RE.test(path)) {
      setError(t("common.invalidPath"));
      return;
    }

    setSending(true);
    setError(null);
    setResponse(null);

    try {
      let parsedBody: Record<string, unknown> | undefined;
      if (body.trim() && ["POST", "PUT", "PATCH"].includes(method)) {
        parsedBody = JSON.parse(body);
      }

      const result = await botApi.proxyMaxApi(botId, {
        method,
        path,
        body: parsedBody,
      });

      setResponse({
        status: result.status,
        data: result.data,
        duration: result.duration,
      });

      saveHistory([
        {
          id: crypto.randomUUID(),
          method,
          path,
          body: "", // Don't persist request body (may contain sensitive data)
          status: result.status,
          duration: result.duration,
          response: "", // Don't persist response (may contain PII)
          timestamp: Date.now(),
        },
        ...history,
      ]);
    } catch (err) {
      if (err instanceof SyntaxError) {
        setError(t("common.invalidJson"));
      } else {
        setError(t("errors.somethingWentWrong"));
      }
    } finally {
      setSending(false);
    }
  };

  const showBody = ["POST", "PUT", "PATCH"].includes(method);

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.2 }}
      className="space-y-4"
    >
      {/* Request builder */}
      <div className="flex gap-2 items-end">
        <div>
          <Label className="text-xs mb-1 block">{t("console.method")}</Label>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm" className={cn("w-24 font-mono font-bold", METHOD_COLORS[method])}>
                {method}
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              {(["GET", "POST", "PUT", "PATCH", "DELETE"] as HttpMethod[]).map(
                (m) => (
                  <DropdownMenuItem
                    key={m}
                    onClick={() => setMethod(m)}
                    className={cn("font-mono font-bold", METHOD_COLORS[m])}
                  >
                    {m}
                  </DropdownMenuItem>
                ),
              )}
            </DropdownMenuContent>
          </DropdownMenu>
        </div>

        <div className="flex-1">
          <Label className="text-xs mb-1 block">{t("console.path")}</Label>
          <div className="relative">
            <Input
              value={path}
              onChange={(e) => setPath(e.target.value)}
              placeholder="/chats"
              className="font-mono text-sm"
              list="path-suggestions"
            />
            <datalist id="path-suggestions">
              {COMMON_PATHS.map((p) => (
                <option key={p} value={p} />
              ))}
            </datalist>
          </div>
        </div>

        <Button
          onClick={handleSend}
          disabled={sending || !path}
          size="sm"
          className="gap-1.5 shrink-0"
        >
          <Play className="size-3.5" />
          {t("console.send")}
        </Button>
      </div>

      {/* Body editor */}
      {showBody && (
        <div>
          <Label className="text-xs mb-1 block">{t("console.body")}</Label>
          <textarea
            value={body}
            onChange={(e) => setBody(e.target.value)}
            placeholder='{"key": "value"}'
            className="w-full h-32 rounded-md border border-input bg-background px-3 py-2 text-sm font-mono resize-y focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          />
        </div>
      )}

      {error && (
        <div className="rounded-md bg-destructive/10 border border-destructive/20 px-3 py-2 text-sm text-destructive">
          {error}
        </div>
      )}

      {/* Response */}
      {response && (
        <Card>
          <CardContent className="p-4 space-y-2">
            <div className="flex items-center gap-2">
              <Badge
                variant={response.status < 400 ? "default" : "destructive"}
                className="font-mono"
              >
                {response.status}
              </Badge>
              {response.duration > 0 && (
                <span className="text-xs text-muted-foreground flex items-center gap-1">
                  <Clock className="size-3" />
                  {response.duration}ms
                </span>
              )}
            </div>
            <JsonViewer data={response.data} maxHeight="50vh" />
          </CardContent>
        </Card>
      )}

      {/* History */}
      {history.length > 0 && (
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium text-muted-foreground">
              {t("console.history")}
            </h3>
            <Button
              variant="ghost"
              size="xs"
              onClick={() => saveHistory([])}
              className="text-muted-foreground"
            >
              <Trash2 className="size-3 mr-1" />
              {t("console.clearHistory")}
            </Button>
          </div>
          <div className="space-y-1">
            {history.slice(0, 10).map((entry) => (
              <button
                key={entry.id ?? entry.timestamp}
                onClick={() => {
                  setMethod(entry.method);
                  setPath(entry.path);
                  setBody(entry.body);
                }}
                className="w-full text-left flex items-center gap-2 px-2 py-1.5 rounded-md hover:bg-muted/50 transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
              >
                <span
                  className={cn(
                    "text-xs font-mono font-bold w-14 shrink-0",
                    METHOD_COLORS[entry.method],
                  )}
                >
                  {entry.method}
                </span>
                <span className="text-xs font-mono truncate flex-1">
                  {entry.path}
                </span>
                {entry.status && (
                  <Badge
                    variant={entry.status < 400 ? "secondary" : "destructive"}
                    className="text-[10px] shrink-0"
                  >
                    {entry.status}
                  </Badge>
                )}
              </button>
            ))}
          </div>
        </div>
      )}
    </motion.div>
  );
}
