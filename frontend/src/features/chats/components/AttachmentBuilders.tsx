import { useTranslation } from "react-i18next";
import {
  X,
  Plus,
  Trash2,
  SmilePlus,
  User,
  MapPin,
  Link2,
  Grid3x3,
  MousePointerClick,
  ExternalLink,
  Contact,
  Navigation,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

// --- Types ---

export type SpecialAttachmentType =
  | "sticker"
  | "contact"
  | "keyboard"
  | "location"
  | "share";

export interface StickerData {
  code: string;
}

export interface ContactData {
  name: string;
  contact_id?: number;
  vcf_phone?: string;
  vcf_info?: string;
}

export type KeyboardButtonType =
  | "callback"
  | "link"
  | "request_contact"
  | "request_geo_location";

export interface KeyboardButton {
  type: KeyboardButtonType;
  text: string;
  payload?: string;
  url?: string;
}

export interface KeyboardData {
  buttons: KeyboardButton[][];
}

export interface LocationData {
  latitude: string;
  longitude: string;
}

export interface ShareData {
  url: string;
}

export type SpecialAttachmentData =
  | StickerData
  | ContactData
  | KeyboardData
  | LocationData
  | ShareData;

/**
 * Build the attachment object for the Max API from the builder data.
 * Returns null if the data is invalid/incomplete.
 */
export function buildAttachment(
  type: SpecialAttachmentType,
  data: SpecialAttachmentData,
): Record<string, unknown> | null {
  switch (type) {
    case "sticker": {
      const d = data as StickerData;
      if (!d.code.trim()) return null;
      return { type: "sticker", payload: { code: d.code.trim() } };
    }
    case "contact": {
      const d = data as ContactData;
      if (!d.name.trim()) return null;
      const payload: Record<string, unknown> = { name: d.name.trim() };
      if (d.contact_id !== undefined) payload.contact_id = d.contact_id;
      if (d.vcf_phone?.trim()) payload.vcf_phone = d.vcf_phone.trim();
      if (d.vcf_info?.trim()) payload.vcf_info = d.vcf_info.trim();
      return { type: "contact", payload };
    }
    case "keyboard": {
      const d = data as KeyboardData;
      const buttons = d.buttons
        .map((row) => row.filter((btn) => btn.text.trim()))
        .filter((row) => row.length > 0);
      if (buttons.length === 0) return null;
      const mapped = buttons.map((row) =>
        row.map((btn) => {
          const b: Record<string, unknown> = {
            type: btn.type,
            text: btn.text.trim(),
          };
          if (btn.type === "callback" && btn.payload)
            b.payload = btn.payload;
          if (btn.type === "link" && btn.url) b.url = btn.url;
          return b;
        }),
      );
      return { type: "inline_keyboard", payload: { buttons: mapped } };
    }
    case "location": {
      const d = data as LocationData;
      const lat = parseFloat(d.latitude);
      const lon = parseFloat(d.longitude);
      if (isNaN(lat) || isNaN(lon)) return null;
      return { type: "location", latitude: lat, longitude: lon };
    }
    case "share": {
      const d = data as ShareData;
      if (!d.url.trim()) return null;
      return { type: "share", payload: { url: d.url.trim() } };
    }
  }
}

// --- Accent color configs per builder type ---

const ACCENTS = {
  sticker: {
    border: "border-amber-500/30",
    bg: "bg-amber-500/5",
    icon: "text-amber-500",
    label: "text-amber-600 dark:text-amber-400",
    dot: "bg-amber-500",
  },
  contact: {
    border: "border-sky-500/30",
    bg: "bg-sky-500/5",
    icon: "text-sky-500",
    label: "text-sky-600 dark:text-sky-400",
    dot: "bg-sky-500",
  },
  keyboard: {
    border: "border-violet-500/30",
    bg: "bg-violet-500/5",
    icon: "text-violet-500",
    label: "text-violet-600 dark:text-violet-400",
    dot: "bg-violet-500",
  },
  location: {
    border: "border-emerald-500/30",
    bg: "bg-emerald-500/5",
    icon: "text-emerald-500",
    label: "text-emerald-600 dark:text-emerald-400",
    dot: "bg-emerald-500",
  },
  share: {
    border: "border-orange-500/30",
    bg: "bg-orange-500/5",
    icon: "text-orange-500",
    label: "text-orange-600 dark:text-orange-400",
    dot: "bg-orange-500",
  },
} as const;

const TYPE_ICONS = {
  sticker: SmilePlus,
  contact: User,
  keyboard: Grid3x3,
  location: MapPin,
  share: Link2,
} as const;

// --- Shared builder wrapper ---

function BuilderPanel({
  type,
  title,
  onDismiss,
  children,
}: {
  type: SpecialAttachmentType;
  title: string;
  onDismiss: () => void;
  children: React.ReactNode;
}) {
  const accent = ACCENTS[type];
  const Icon = TYPE_ICONS[type];

  return (
    <div
      className={cn(
        "mb-2 rounded-lg border overflow-hidden",
        accent.border,
        accent.bg,
      )}
    >
      <div className="flex items-center gap-2 px-3 py-1.5">
        <Icon className={cn("size-3.5 shrink-0", accent.icon)} />
        <span className={cn("text-[11px] font-semibold tracking-wide uppercase", accent.label)}>
          {title}
        </span>
        <div className="flex-1" />
        <button
          onClick={onDismiss}
          className="rounded-sm p-0.5 hover:bg-foreground/5 transition-colors"
        >
          <X className="size-3.5 text-muted-foreground" />
        </button>
      </div>
      <div className="px-3 pb-2.5">{children}</div>
    </div>
  );
}

// --- Sticker Builder ---

export function StickerBuilder({
  data,
  onChange,
  onDismiss,
}: {
  data: StickerData;
  onChange: (d: StickerData) => void;
  onDismiss: () => void;
}) {
  const { t } = useTranslation();
  return (
    <BuilderPanel
      type="sticker"
      title={t("chats.attachments.sticker")}
      onDismiss={onDismiss}
    >
      <div className="flex items-center gap-2">
        <div className="size-10 rounded-lg bg-amber-500/10 border border-amber-500/20 flex items-center justify-center shrink-0">
          <SmilePlus className="size-5 text-amber-500/60" />
        </div>
        <Input
          value={data.code}
          onChange={(e) => onChange({ code: e.target.value })}
          placeholder={t("chats.compose.stickerCode")}
          className="h-8 text-xs font-mono bg-background/80"
          autoFocus
        />
      </div>
    </BuilderPanel>
  );
}

// --- Contact Builder ---

export function ContactBuilder({
  data,
  onChange,
  onDismiss,
}: {
  data: ContactData;
  onChange: (d: ContactData) => void;
  onDismiss: () => void;
}) {
  const { t } = useTranslation();
  return (
    <BuilderPanel
      type="contact"
      title={t("chats.attachments.contact")}
      onDismiss={onDismiss}
    >
      <div className="flex gap-2.5">
        {/* Contact avatar placeholder */}
        <div className="size-12 rounded-full bg-sky-500/10 border border-sky-500/20 flex items-center justify-center shrink-0 mt-0.5">
          <Contact className="size-5 text-sky-500/60" />
        </div>
        <div className="flex-1 min-w-0 space-y-1.5">
          <Input
            value={data.name}
            onChange={(e) => onChange({ ...data, name: e.target.value })}
            placeholder={t("chats.compose.contactName") + " *"}
            className="h-7 text-xs bg-background/80 font-medium"
            autoFocus
          />
          <div className="grid grid-cols-2 gap-1.5">
            <Input
              type="number"
              value={data.contact_id ?? ""}
              onChange={(e) =>
                onChange({
                  ...data,
                  contact_id: e.target.value
                    ? Number(e.target.value)
                    : undefined,
                })
              }
              placeholder={t("chats.compose.contactId")}
              className="h-7 text-xs bg-background/80"
            />
            <Input
              value={data.vcf_phone ?? ""}
              onChange={(e) =>
                onChange({ ...data, vcf_phone: e.target.value || undefined })
              }
              placeholder={t("chats.compose.contactPhone")}
              className="h-7 text-xs bg-background/80"
            />
          </div>
          <Input
            value={data.vcf_info ?? ""}
            onChange={(e) =>
              onChange({ ...data, vcf_info: e.target.value || undefined })
            }
            placeholder={t("chats.compose.contactVcf")}
            className="h-7 text-xs bg-background/80"
          />
        </div>
      </div>
    </BuilderPanel>
  );
}

// --- Inline Keyboard Builder ---

const BUTTON_TYPE_CONFIG: Record<
  KeyboardButtonType,
  { label: string; icon: typeof MousePointerClick; color: string }
> = {
  callback: { label: "Callback", icon: MousePointerClick, color: "text-violet-400" },
  link: { label: "Link", icon: ExternalLink, color: "text-blue-400" },
  request_contact: { label: "Contact", icon: Contact, color: "text-cyan-400" },
  request_geo_location: { label: "Geo", icon: Navigation, color: "text-green-400" },
};

function KeyboardButtonEditor({
  button,
  onChange,
  onDelete,
}: {
  button: KeyboardButton;
  onChange: (b: KeyboardButton) => void;
  onDelete: () => void;
}) {
  const { t } = useTranslation();
  const config = BUTTON_TYPE_CONFIG[button.type];

  return (
    <div className="rounded-lg border border-violet-500/20 bg-background p-2 space-y-1.5">
      <div className="flex gap-1.5">
        <Input
          value={button.text}
          onChange={(e) => onChange({ ...button, text: e.target.value })}
          placeholder={t("chats.compose.buttonText")}
          className="h-7 text-xs flex-1"
        />
        <Button
          variant="ghost"
          size="icon-xs"
          onClick={onDelete}
          className="shrink-0"
        >
          <Trash2 className="size-3 text-muted-foreground" />
        </Button>
      </div>
      <div className="flex gap-1">
        {(Object.entries(BUTTON_TYPE_CONFIG) as [KeyboardButtonType, typeof config][]).map(
          ([val, cfg]) => {
            const BtnIcon = cfg.icon;
            return (
              <button
                key={val}
                type="button"
                onClick={() =>
                  onChange({
                    ...button,
                    type: val,
                    payload: undefined,
                    url: undefined,
                  })
                }
                className={cn(
                  "flex items-center gap-1 rounded px-1.5 py-0.5 text-[10px] transition-colors border",
                  button.type === val
                    ? "bg-violet-500/10 border-violet-500/30 text-violet-600 dark:text-violet-400 font-medium"
                    : "border-transparent hover:bg-accent text-muted-foreground",
                )}
              >
                <BtnIcon className="size-2.5" />
                {cfg.label}
              </button>
            );
          },
        )}
      </div>
      {button.type === "callback" && (
        <Input
          value={button.payload ?? ""}
          onChange={(e) => onChange({ ...button, payload: e.target.value })}
          placeholder={t("chats.compose.buttonPayload")}
          maxLength={1024}
          className="h-7 text-xs font-mono"
        />
      )}
      {button.type === "link" && (
        <Input
          value={button.url ?? ""}
          onChange={(e) => onChange({ ...button, url: e.target.value })}
          placeholder={t("chats.compose.buttonUrl")}
          maxLength={2048}
          className="h-7 text-xs"
        />
      )}
    </div>
  );
}

export function KeyboardBuilder({
  data,
  onChange,
  onDismiss,
}: {
  data: KeyboardData;
  onChange: (d: KeyboardData) => void;
  onDismiss: () => void;
}) {
  const { t } = useTranslation();
  const updateButton = (rowIdx: number, btnIdx: number, btn: KeyboardButton) => {
    const rows = data.buttons.map((r) => [...r]);
    rows[rowIdx][btnIdx] = btn;
    onChange({ buttons: rows });
  };

  const deleteButton = (rowIdx: number, btnIdx: number) => {
    const rows = data.buttons.map((r) => [...r]);
    rows[rowIdx].splice(btnIdx, 1);
    onChange({ buttons: rows.filter((r) => r.length > 0) });
  };

  const addButton = (rowIdx: number) => {
    const rows = data.buttons.map((r) => [...r]);
    rows[rowIdx].push({ type: "callback", text: "", payload: "" });
    onChange({ buttons: rows });
  };

  const addRow = () => {
    onChange({
      buttons: [
        ...data.buttons,
        [{ type: "callback", text: "", payload: "" }],
      ],
    });
  };

  const deleteRow = (rowIdx: number) => {
    onChange({ buttons: data.buttons.filter((_, i) => i !== rowIdx) });
  };

  return (
    <BuilderPanel
      type="keyboard"
      title={t("chats.compose.keyboardBuilder")}
      onDismiss={onDismiss}
    >
      <div className="space-y-2 max-h-56 overflow-y-auto pr-0.5">
        {data.buttons.map((row, rowIdx) => (
          <div key={rowIdx}>
            {/* Row header */}
            <div className="flex items-center gap-1 mb-1">
              <span className="text-[10px] font-medium text-violet-500/60 tabular-nums">
                {t("chats.compose.addRow").replace(/\s.*/, "")} {rowIdx + 1}
              </span>
              <div className="flex-1 border-t border-violet-500/10" />
              <button
                type="button"
                onClick={() => addButton(rowIdx)}
                className="rounded p-0.5 hover:bg-violet-500/10 transition-colors"
                title={t("chats.compose.addButton")}
              >
                <Plus className="size-3 text-violet-500/60" />
              </button>
              {data.buttons.length > 1 && (
                <button
                  type="button"
                  onClick={() => deleteRow(rowIdx)}
                  className="rounded p-0.5 hover:bg-destructive/10 transition-colors"
                >
                  <Trash2 className="size-3 text-muted-foreground/40" />
                </button>
              )}
            </div>
            {/* Button chips in a row */}
            <div className="flex flex-wrap gap-1.5">
              {row.map((btn, btnIdx) => (
                <KeyboardButtonEditor
                  key={btnIdx}
                  button={btn}
                  onChange={(b) => updateButton(rowIdx, btnIdx, b)}
                  onDelete={() => deleteButton(rowIdx, btnIdx)}
                />
              ))}
            </div>
          </div>
        ))}
      </div>
      <button
        type="button"
        onClick={addRow}
        className="mt-2 w-full flex items-center justify-center gap-1.5 rounded-md border border-dashed border-violet-500/20 py-1.5 text-[11px] text-violet-500/60 hover:text-violet-500 hover:border-violet-500/40 hover:bg-violet-500/5 transition-all"
      >
        <Plus className="size-3" />
        {t("chats.compose.addRow")}
      </button>
    </BuilderPanel>
  );
}

// --- Location Builder ---

export function LocationBuilder({
  data,
  onChange,
  onDismiss,
}: {
  data: LocationData;
  onChange: (d: LocationData) => void;
  onDismiss: () => void;
}) {
  const { t } = useTranslation();
  return (
    <BuilderPanel
      type="location"
      title={t("chats.attachments.location")}
      onDismiss={onDismiss}
    >
      <div className="flex items-center gap-2">
        {/* Crosshair visual */}
        <div className="size-10 rounded-lg bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center shrink-0 relative">
          <MapPin className="size-5 text-emerald-500/60" />
          <div className="absolute inset-0 rounded-lg ring-1 ring-emerald-500/10 ring-offset-1 ring-offset-transparent" />
        </div>
        <div className="flex-1 grid grid-cols-2 gap-1.5">
          <div>
            <label className="text-[10px] text-emerald-600/60 dark:text-emerald-400/60 font-medium mb-0.5 block">
              {t("chats.compose.latitude")}
            </label>
            <Input
              type="number"
              step="any"
              value={data.latitude}
              onChange={(e) => onChange({ ...data, latitude: e.target.value })}
              placeholder="55.7558"
              className="h-7 text-xs font-mono bg-background/80"
              autoFocus
            />
          </div>
          <div>
            <label className="text-[10px] text-emerald-600/60 dark:text-emerald-400/60 font-medium mb-0.5 block">
              {t("chats.compose.longitude")}
            </label>
            <Input
              type="number"
              step="any"
              value={data.longitude}
              onChange={(e) => onChange({ ...data, longitude: e.target.value })}
              placeholder="37.6173"
              className="h-7 text-xs font-mono bg-background/80"
            />
          </div>
        </div>
      </div>
    </BuilderPanel>
  );
}

// --- Share Builder ---

export function ShareBuilder({
  data,
  onChange,
  onDismiss,
}: {
  data: ShareData;
  onChange: (d: ShareData) => void;
  onDismiss: () => void;
}) {
  const { t } = useTranslation();
  return (
    <BuilderPanel
      type="share"
      title={t("chats.attachments.share")}
      onDismiss={onDismiss}
    >
      <div className="flex items-center gap-2">
        <div className="size-10 rounded-lg bg-orange-500/10 border border-orange-500/20 flex items-center justify-center shrink-0">
          <Link2 className="size-5 text-orange-500/60" />
        </div>
        <Input
          value={data.url}
          onChange={(e) => onChange({ url: e.target.value })}
          placeholder={t("chats.compose.shareUrl")}
          className="h-8 text-xs bg-background/80"
          autoFocus
        />
      </div>
    </BuilderPanel>
  );
}

// --- Default data factory ---

export function defaultDataFor(
  type: SpecialAttachmentType,
): SpecialAttachmentData {
  switch (type) {
    case "sticker":
      return { code: "" } satisfies StickerData;
    case "contact":
      return { name: "" } satisfies ContactData;
    case "keyboard":
      return {
        buttons: [[{ type: "callback", text: "", payload: "" }]],
      } satisfies KeyboardData;
    case "location":
      return { latitude: "", longitude: "" } satisfies LocationData;
    case "share":
      return { url: "" } satisfies ShareData;
  }
}

// --- Dispatcher component ---

export function AttachmentBuilder({
  type,
  data,
  onChange,
  onDismiss,
}: {
  type: SpecialAttachmentType;
  data: SpecialAttachmentData;
  onChange: (d: SpecialAttachmentData) => void;
  onDismiss: () => void;
}) {
  switch (type) {
    case "sticker":
      return (
        <StickerBuilder
          data={data as StickerData}
          onChange={onChange}
          onDismiss={onDismiss}
        />
      );
    case "contact":
      return (
        <ContactBuilder
          data={data as ContactData}
          onChange={onChange}
          onDismiss={onDismiss}
        />
      );
    case "keyboard":
      return (
        <KeyboardBuilder
          data={data as KeyboardData}
          onChange={onChange}
          onDismiss={onDismiss}
        />
      );
    case "location":
      return (
        <LocationBuilder
          data={data as LocationData}
          onChange={onChange}
          onDismiss={onDismiss}
        />
      );
    case "share":
      return (
        <ShareBuilder
          data={data as ShareData}
          onChange={onChange}
          onDismiss={onDismiss}
        />
      );
  }
}
