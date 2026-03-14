import { useMemo } from "react";
import { Copy } from "lucide-react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { Button } from "./button";
import { cn } from "@/lib/utils";

interface JsonViewerProps {
  data: unknown;
  maxHeight?: string;
  className?: string;
  copyable?: boolean;
}

function colorize(json: string): React.ReactNode[] {
  const parts: React.ReactNode[] = [];
  let i = 0;

  // Simple regex-based colorization
  const regex = /("(?:[^"\\]|\\.)*")\s*:|("(?:[^"\\]|\\.)*")|(true|false|null)|(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = regex.exec(json)) !== null) {
    // Text before match
    if (match.index > lastIndex) {
      parts.push(
        <span key={i++}>{json.slice(lastIndex, match.index)}</span>
      );
    }

    if (match[1]) {
      // Key
      parts.push(
        <span key={i++} className="text-primary/80">{match[1]}</span>
      );
      parts.push(<span key={i++}>:</span>);
    } else if (match[2]) {
      // String value
      parts.push(
        <span key={i++} className="text-emerald-600 dark:text-emerald-400">{match[2]}</span>
      );
    } else if (match[3]) {
      // Boolean/null
      parts.push(
        <span key={i++} className="text-amber-600 dark:text-amber-400">{match[3]}</span>
      );
    } else if (match[4]) {
      // Number
      parts.push(
        <span key={i++} className="text-blue-600 dark:text-blue-400">{match[4]}</span>
      );
    }

    lastIndex = match.index + match[0].length;
  }

  // Remaining text
  if (lastIndex < json.length) {
    parts.push(<span key={i++}>{json.slice(lastIndex)}</span>);
  }

  return parts;
}

export function JsonViewer({
  data,
  maxHeight = "60vh",
  className,
  copyable = true,
}: JsonViewerProps) {
  const { t } = useTranslation();
  const jsonStr = useMemo(() => JSON.stringify(data, null, 2), [data]);
  const colorized = useMemo(() => colorize(jsonStr), [jsonStr]);

  return (
    <div className={cn("relative group", className)}>
      {copyable && (
        <Button
          variant="ghost"
          size="icon-xs"
          className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 focus-visible:opacity-100 transition-opacity"
          onClick={() => {
            navigator.clipboard.writeText(jsonStr);
            toast.success(t("common.copied"));
          }}
        >
          <Copy className="size-3" />
        </Button>
      )}
      <pre
        className={cn(
          "text-xs font-mono bg-muted/50 rounded-md p-3 overflow-x-auto whitespace-pre-wrap",
        )}
        style={{ maxHeight }}
      >
        {colorized}
      </pre>
    </div>
  );
}
