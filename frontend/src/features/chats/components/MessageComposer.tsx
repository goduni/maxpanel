import { useCallback as useCb, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Send,
  X,
  Reply,
  Paperclip,
  Loader2,
  Plus,
  SmilePlus,
  User,
  Keyboard,
  MapPin,
  Link,
  Link2,
  Bold,
  Italic,
  Strikethrough,
  Underline,
  Code,
} from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { proxyMaxApi } from "@/features/bots/api";
import {
  AttachmentBuilder,
  buildAttachment,
  defaultDataFor,
  type SpecialAttachmentType,
  type SpecialAttachmentData,
} from "./AttachmentBuilders";

type FormatStyle = "strong" | "emphasized" | "monospaced" | "strikethrough" | "underline" | "link";

interface ReplyTo {
  mid: string;
  senderName: string;
  text: string;
}

interface SpecialAttachment {
  id: number;
  type: SpecialAttachmentType;
  data: SpecialAttachmentData;
}

interface MessageComposerProps {
  botId: string;
  chatId: number;
  replyTo: ReplyTo | null;
  onClearReply: () => void;
  onSent: () => void;
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function getUploadType(
  mimeType: string,
): "image" | "video" | "audio" | "file" {
  if (mimeType.startsWith("image/")) return "image";
  if (mimeType.startsWith("video/")) return "video";
  if (mimeType.startsWith("audio/")) return "audio";
  return "file";
}

type TextFormat = "markdown" | "html";

const FORMAT_SYNTAX: Record<TextFormat, Record<FormatStyle, [string, string]>> = {
  markdown: {
    strong: ["**", "**"],
    emphasized: ["_", "_"],
    monospaced: ["`", "`"],
    strikethrough: ["~~", "~~"],
    underline: ["++", "++"],
    link: ["[", "](url)"],
  },
  html: {
    strong: ["<b>", "</b>"],
    emphasized: ["<i>", "</i>"],
    monospaced: ["<code>", "</code>"],
    strikethrough: ["<s>", "</s>"],
    underline: ["<u>", "</u>"],
    link: ['<a href="url">', "</a>"],
  },
};


let nextSpecialId = 0;

export function MessageComposer({
  botId,
  chatId,
  replyTo,
  onClearReply,
  onSent,
}: MessageComposerProps) {
  const { t } = useTranslation();
  const [text, setText] = useState("");
  const [format, setFormat] = useState<TextFormat>("markdown");
  const [formatted, setFormatted] = useState(false);
  const [sending, setSending] = useState(false);
  const [files, setFiles] = useState<File[]>([]);
  const [specials, setSpecials] = useState<SpecialAttachment[]>([]);
  const [hasSelection, setHasSelection] = useState(false);
  const [toolbarPos, setToolbarPos] = useState<{ top: number; left: number } | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const wrapperRef = useRef<HTMLDivElement>(null);

  // Track text selection in textarea (debounced to avoid flicker on mobile keyboards)
  const selectionTimer = useRef<ReturnType<typeof setTimeout>>(null);
  const checkSelection = useCb(() => {
    if (selectionTimer.current) clearTimeout(selectionTimer.current);
    selectionTimer.current = setTimeout(() => {
      const el = textareaRef.current;
      if (!el || document.activeElement !== el || el.selectionStart === el.selectionEnd) {
        setHasSelection(false);
        setToolbarPos(null);
        return;
      }
      setHasSelection(true);
      // Position relative to wrapper (absolute, not fixed)
      setToolbarPos({ top: -36, left: 0 });
    }, 150);
  }, []);

  useEffect(() => {
    document.addEventListener("selectionchange", checkSelection);
    return () => {
      document.removeEventListener("selectionchange", checkSelection);
      if (selectionTimer.current) clearTimeout(selectionTimer.current);
    };
  }, [checkSelection]);

  const uploadFile = async (
    file: File,
  ): Promise<{ type: string; token: string } | null> => {
    const uploadType = getUploadType(file.type);

    const urlResult = await proxyMaxApi(botId, {
      method: "POST",
      path: `/uploads?type=${uploadType}`,
    });

    if (urlResult.status < 200 || urlResult.status >= 300) return null;
    const uploadUrl = (urlResult.data as Record<string, unknown>)?.url;
    if (typeof uploadUrl !== "string") return null;

    const formData = new FormData();
    formData.append("data", file);
    const uploadRes = await fetch(uploadUrl, {
      method: "POST",
      body: formData,
    });
    if (!uploadRes.ok) return null;
    const uploadData = (await uploadRes.json()) as Record<string, unknown>;

    let token: string | null = null;
    if (uploadType === "image" && uploadData.photos) {
      const photos = uploadData.photos as Record<
        string,
        Record<string, unknown>
      >;
      const firstPhoto = Object.values(photos)[0];
      token =
        typeof firstPhoto?.token === "string" ? firstPhoto.token : null;
    } else {
      token = typeof uploadData.token === "string" ? uploadData.token : null;
    }

    if (!token) return null;
    return { type: uploadType, token };
  };

  const addSpecial = (type: SpecialAttachmentType) => {
    setSpecials((prev) => [
      ...prev,
      { id: ++nextSpecialId, type, data: defaultDataFor(type) },
    ]);
  };

  const updateSpecial = (id: number, data: SpecialAttachmentData) => {
    setSpecials((prev) =>
      prev.map((s) => (s.id === id ? { ...s, data } : s)),
    );
  };

  const removeSpecial = (id: number) => {
    setSpecials((prev) => prev.filter((s) => s.id !== id));
  };

  const removeFile = (index: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== index));
  };

  /** Check if an API error is "attachment not ready" (file still processing). */
  const isAttachmentNotReady = (result: { data: unknown }): boolean => {
    const errData = result.data as Record<string, unknown> | null;
    const upstream = (errData?.error as Record<string, unknown>)?.upstream as Record<string, unknown> | undefined;
    return upstream?.code === "attachment.not.ready";
  };

  /** Extract error message from proxy result. */
  const getErrorMessage = (result: { data: unknown }): string => {
    const errData = result.data as Record<string, unknown> | null;
    const errObj = errData?.error as Record<string, unknown> | undefined;
    const msg = typeof errObj?.message === "string" ? errObj.message : t("errors.somethingWentWrong");
    const upstream = errObj?.upstream ? `\n${JSON.stringify(errObj.upstream, null, 2)}` : "";
    return `${msg}${upstream}`;
  };

  /** Send message with retry for "attachment not ready" errors. */
  const sendMessage = async (
    body: Record<string, unknown>,
  ): Promise<{ ok: boolean; retryable: boolean; error?: string }> => {
    const MAX_RETRIES = 5;
    const RETRY_DELAY = 2000;

    for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
      const result = await proxyMaxApi(botId, {
        method: "POST",
        path: `/messages?chat_id=${chatId}`,
        body,
      });

      if (result.status >= 200 && result.status < 300) {
        return { ok: true, retryable: false };
      }

      if (isAttachmentNotReady(result) && attempt < MAX_RETRIES) {
        await new Promise((r) => setTimeout(r, RETRY_DELAY));
        continue;
      }

      return {
        ok: false,
        retryable: isAttachmentNotReady(result),
        error: getErrorMessage(result),
      };
    }

    return { ok: false, retryable: true, error: t("errors.somethingWentWrong") };
  };

  // Keep last built body for manual retry
  const lastBodyRef = useRef<Record<string, unknown> | null>(null);

  /** Wrap selected text with formatting syntax directly in the textarea. */
  const applyFormat = (style: FormatStyle) => {
    const el = textareaRef.current;
    if (!el) return;
    const start = el.selectionStart;
    const end = el.selectionEnd;
    if (start === end) return;

    const selected = text.slice(start, end);
    const [open, close] = FORMAT_SYNTAX[format][style];

    let replacement: string;
    if (style === "link") {
      const url = prompt(t("chats.compose.enterUrl", "Enter URL:"));
      if (!url) return;
      replacement = format === "markdown"
        ? `[${selected}](${url})`
        : `<a href="${url}">${selected}</a>`;
    } else {
      replacement = open + selected + close;
    }

    const newText = text.slice(0, start) + replacement + text.slice(end);
    setText(newText);
    setFormatted(true);

    // Move cursor after the inserted text
    const newCursorPos = start + replacement.length;
    requestAnimationFrame(() => {
      el.focus();
      el.setSelectionRange(newCursorPos, newCursorPos);
      // Auto-resize
      el.style.height = "auto";
      const maxH = Math.min(160, window.innerHeight * 0.25);
    el.style.height = Math.min(el.scrollHeight, maxH) + "px";
    });
  };

  const handleSend = async () => {
    const trimmed = text.trim();
    if ((!trimmed && files.length === 0 && specials.length === 0) || sending)
      return;

    setSending(true);
    try {
      const body: Record<string, unknown> = {};
      if (trimmed) {
        body.text = trimmed;
        if (formatted) {
          body.format = format;
        }
      }
      if (replyTo) {
        body.link = { type: "reply", mid: replyTo.mid };
      }

      const attachments: Record<string, unknown>[] = [];

      // Upload all files
      for (const file of files) {
        const uploaded = await uploadFile(file);
        if (!uploaded) {
          toast.error(
            `${t("errors.somethingWentWrong")}: ${file.name}`,
            { duration: 10000 },
          );
          setSending(false);
          return;
        }
        attachments.push({
          type: uploaded.type,
          payload: { token: uploaded.token },
        });
      }

      // Build all special attachments
      for (const special of specials) {
        const built = buildAttachment(special.type, special.data);
        if (!built) {
          toast.error(t("errors.validationError"), { duration: 10000 });
          setSending(false);
          return;
        }
        attachments.push(built);
      }

      if (attachments.length > 0) {
        body.attachments = attachments;
      }

      lastBodyRef.current = body;
      const result = await sendMessage(body);

      if (result.ok) {
        lastBodyRef.current = null;
        setText("");
        setFormatted(false);
        setFiles([]);
        setSpecials([]);
        onClearReply();
        onSent();
        if (textareaRef.current) {
          textareaRef.current.style.height = "auto";
          textareaRef.current.focus();
        }
      } else if (result.retryable) {
        // Attachment still processing after retries — offer manual retry
        toast.error(
          t("chats.compose.attachmentProcessing", "File is still being processed. Try again in a few seconds."),
          {
            duration: 30000,
            action: {
              label: t("common.retry", "Retry"),
              onClick: () => handleRetrySend(),
            },
          },
        );
      } else {
        toast.error(result.error ?? t("errors.somethingWentWrong"), { duration: 10000 });
      }
    } catch {
      toast.error(t("errors.somethingWentWrong"), { duration: 10000 });
    } finally {
      setSending(false);
    }
  };

  const handleRetrySend = async () => {
    const body = lastBodyRef.current;
    if (!body || sending) return;
    setSending(true);
    try {
      const result = await sendMessage(body);
      if (result.ok) {
        lastBodyRef.current = null;
        setText("");
        setFormatted(false);
        setFiles([]);
        setSpecials([]);
        onClearReply();
        onSent();
        if (textareaRef.current) {
          textareaRef.current.style.height = "auto";
          textareaRef.current.focus();
        }
      } else if (result.retryable) {
        toast.error(
          t("chats.compose.attachmentProcessing", "File is still being processed. Try again in a few seconds."),
          {
            duration: 30000,
            action: {
              label: t("common.retry", "Retry"),
              onClick: () => handleRetrySend(),
            },
          },
        );
      } else {
        toast.error(result.error ?? t("errors.somethingWentWrong"), { duration: 10000 });
      }
    } catch {
      toast.error(t("errors.somethingWentWrong"), { duration: 10000 });
    } finally {
      setSending(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setText(e.target.value);
    const el = e.target;
    el.style.height = "auto";
    const maxH = Math.min(160, window.innerHeight * 0.25);
    el.style.height = Math.min(el.scrollHeight, maxH) + "px";
  };

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selected = e.target.files;
    if (!selected || selected.length === 0) return;
    // Copy files BEFORE resetting input (some browsers clear FileList on reset)
    const newFiles = Array.from(selected);
    e.target.value = "";
    setFiles((prev) => [...prev, ...newFiles]);
  };


  const canSend =
    (text.trim() || files.length > 0 || specials.length > 0) && !sending;

  return (
    <div className="shrink-0 border-t border-border pt-3 mt-2">
      {/* Reply indicator */}
      {replyTo && (
        <div className="flex items-center gap-2 mb-2 px-1">
          <Reply className="size-3.5 text-primary shrink-0" />
          <div className="flex-1 min-w-0 border-l-2 border-primary pl-2">
            <p className="text-xs font-medium text-primary truncate">
              {replyTo.senderName}
            </p>
            <p className="text-xs text-muted-foreground truncate">
              {replyTo.text}
            </p>
          </div>
          <Button
            variant="ghost"
            size="icon-xs"
            onClick={onClearReply}
            aria-label={t("common.cancel")}
          >
            <X className="size-3.5" />
          </Button>
        </div>
      )}

      {/* File previews */}
      {files.length > 0 && (
        <div className="flex flex-wrap gap-1.5 mb-2 px-1">
          {files.map((file, i) => (
            <div
              key={`${file.name}-${i}`}
              className="flex items-center gap-1.5 rounded-md border border-border bg-muted/50 px-2.5 py-1.5 text-xs min-w-0"
            >
              <Paperclip className="size-3.5 shrink-0 text-muted-foreground" />
              <span className="truncate max-w-24 sm:max-w-32 md:max-w-48">{file.name}</span>
              <span className="text-muted-foreground shrink-0">
                ({formatFileSize(file.size)})
              </span>
              <button
                type="button"
                onClick={() => removeFile(i)}
                className="ml-0.5 shrink-0 rounded-sm p-0.5 hover:bg-accent transition-colors"
                aria-label={t("common.cancel")}
              >
                <X className="size-3" />
              </button>
            </div>
          ))}
        </div>
      )}

      {/* Special attachment builder panels */}
      {specials.map((special) => (
        <div key={special.id} className="px-1">
          <AttachmentBuilder
            type={special.type}
            data={special.data}
            onChange={(data) => updateSpecial(special.id, data)}
            onDismiss={() => removeSpecial(special.id)}
          />
        </div>
      ))}

      <div className="flex items-end gap-2">
        {/* Attachment menu */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              disabled={sending}
              aria-label={t("chats.compose.attachments")}
              className="shrink-0"
            >
              <Plus className="size-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start" className="min-w-44">
            <DropdownMenuItem
              onClick={() => fileInputRef.current?.click()}
            >
              <Paperclip className="size-4 mr-2" />
              {t("chats.compose.attachFile")}
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem onClick={() => addSpecial("sticker")}>
              <SmilePlus className="size-4 mr-2" />
              {t("chats.attachments.sticker")}
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => addSpecial("contact")}>
              <User className="size-4 mr-2" />
              {t("chats.attachments.contact")}
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => addSpecial("keyboard")}>
              <Keyboard className="size-4 mr-2" />
              {t("chats.compose.keyboardBuilder")}
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => addSpecial("location")}>
              <MapPin className="size-4 mr-2" />
              {t("chats.attachments.location")}
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => addSpecial("share")}>
              <Link className="size-4 mr-2" />
              {t("chats.attachments.share")}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
        <input
          ref={fileInputRef}
          type="file"
          className="hidden"
          multiple
          onChange={handleFileSelect}
        />

        <div className="flex-1 min-w-0 relative" ref={wrapperRef}>
          <textarea
            ref={textareaRef}
            value={text}
            onChange={handleInput}
            onKeyDown={handleKeyDown}
            onSelect={checkSelection}
            onTouchEnd={checkSelection}
            placeholder={t("chats.compose.placeholder", "Введите сообщение...")}
            rows={1}
            className="w-full resize-none rounded-lg border border-input bg-background px-3 py-2 text-sm shadow-xs placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50"
            disabled={sending}
          />
        </div>

        {/* Floating formatting toolbar — appears on text selection */}
        {hasSelection && toolbarPos && (
          <div
            className="absolute z-50 flex items-center gap-0.5 rounded-lg border border-border bg-popover px-1.5 py-1 shadow-lg left-0 right-0 mx-auto w-fit"
            style={{ top: toolbarPos.top }}
          >
            {([
              { style: "strong" as FormatStyle, icon: Bold, label: "Bold" },
              { style: "emphasized" as FormatStyle, icon: Italic, label: "Italic" },
              { style: "strikethrough" as FormatStyle, icon: Strikethrough, label: "Strikethrough" },
              { style: "underline" as FormatStyle, icon: Underline, label: "Underline" },
              { style: "monospaced" as FormatStyle, icon: Code, label: "Code" },
              { style: "link" as FormatStyle, icon: Link2, label: "Link" },
            ]).map(({ style, icon: Icon, label }) => (
              <button
                key={style}
                type="button"
                onMouseDown={(e) => {
                  e.preventDefault();
                  applyFormat(style);
                }}
                className="rounded p-1.5 sm:p-1.5 min-w-[36px] min-h-[36px] flex items-center justify-center text-popover-foreground hover:bg-accent active:bg-accent transition-colors"
                title={label}
              >
                <Icon className="size-3.5" />
              </button>
            ))}
            <div className="w-px h-4 bg-border mx-0.5" />
            <button
              type="button"
              onMouseDown={(e) => {
                e.preventDefault();
                setFormat((f) => (f === "markdown" ? "html" : "markdown"));
              }}
              className="rounded px-1.5 py-0.5 text-[10px] font-mono text-popover-foreground hover:bg-accent transition-colors"
              title={`Format: ${format}`}
            >
              {format === "markdown" ? "MD" : "HTML"}
            </button>
          </div>
        )}

        <Button
          size="icon"
          onClick={handleSend}
          disabled={!canSend}
          aria-label={t("chats.compose.send", "Отправить")}
        >
          {sending ? (
            <Loader2 className="size-4 animate-spin" />
          ) : (
            <Send className="size-4" />
          )}
        </Button>
      </div>
    </div>
  );
}

export type { ReplyTo };
