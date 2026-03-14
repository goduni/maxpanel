import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  AlertCircle,
  FileText,
  Hash,
  Image,
  MapPin,
  Mic,
  Paperclip,
  Sticker,
  Users,
  Video,
} from "lucide-react";
import { VideoPlayer } from "./VideoPlayer";

const ATTACHMENT_ICONS: Record<string, React.ElementType> = {
  image: Image,
  video: Video,
  audio: Mic,
  file: FileText,
  sticker: Sticker,
  location: MapPin,
  contact: Users,
  share: Paperclip,
  inline_keyboard: Hash,
};

export function AttachmentChip({
  attachment,
}: {
  attachment: Record<string, unknown>;
}) {
  const { t } = useTranslation();
  const type = typeof attachment.type === "string" ? attachment.type : "file";
  const Icon = ATTACHMENT_ICONS[type] ?? Paperclip;

  // Try to get useful info
  const LABEL_KEYS: Record<string, string> = {
    image: "chats.attachments.image",
    video: "chats.attachments.video",
    audio: "chats.attachments.audio",
    file: "chats.attachments.file",
    sticker: "chats.attachments.sticker",
    location: "chats.attachments.location",
    contact: "chats.attachments.contact",
    share: "chats.attachments.share",
    inline_keyboard: "chats.attachments.inline_keyboard",
  };
  let label = t(LABEL_KEYS[type] ?? "chats.attachments.file");
  // Override with content-specific labels where available
  if (type === "audio") {
    const transcription = attachment.transcription;
    if (typeof transcription === "string") label = transcription;
  }
  if (type === "file") {
    const filename = attachment.filename;
    if (typeof filename === "string") label = filename;
  }
  if (type === "share") {
    const title = attachment.title;
    if (typeof title === "string") label = title;
  }

  // Image preview
  if (type === "image") {
    const payload = attachment.payload as Record<string, unknown> | undefined;
    const url = payload && typeof payload.url === "string" ? payload.url : null;
    if (url) {
      return (
        <img
          src={url}
          alt=""
          referrerPolicy="no-referrer"
          className="max-h-48 max-w-xs rounded-md border border-border object-cover"
        />
      );
    }
  }

  // Video preview -- fetch fresh URL via token (Max URLs expire)
  if (type === "video") {
    const payload = attachment.payload as Record<string, unknown> | undefined;
    const token =
      payload && typeof payload.token === "string" ? payload.token : null;
    if (token) {
      return <VideoPlayer token={token} />;
    }
  }

  // Audio player
  if (type === "audio") {
    const payload = attachment.payload as Record<string, unknown> | undefined;
    const url = payload && typeof payload.url === "string" ? payload.url : null;
    if (url) {
      return <AudioPlayer url={url} />;
    }
  }

  return (
    <span className="inline-flex items-center gap-1 text-xs text-muted-foreground bg-muted rounded-md px-2 py-1">
      <Icon className="size-3" />
      <span className="truncate max-w-[150px]">{label}</span>
    </span>
  );
}

function AudioPlayer({ url }: { url: string }) {
  const { t } = useTranslation();
  const [error, setError] = useState(false);

  if (error) {
    return (
      <button
        onClick={() => setError(false)}
        className="inline-flex items-center gap-1 text-xs text-muted-foreground bg-muted rounded-md px-2 py-1 hover:bg-muted/80 transition-colors"
      >
        <AlertCircle className="size-3 text-destructive" />
        <span>{t("errors.mediaLoadFailed")}</span>
      </button>
    );
  }

  return (
    <audio
      ref={(el) => {
        if (el) el.setAttribute("referrerpolicy", "no-referrer");
      }}
      src={url}
      controls
      onError={() => setError(true)}
      className="h-8 max-w-xs"
    />
  );
}
