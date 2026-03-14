import { Link, useLocation, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  Bot,
  Building2,
  FolderKanban,
  Inbox,
  Menu,
  MessageSquare,
  ScrollText,
  Settings,
  Terminal,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import {
  Sheet,
  SheetContent,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { useState } from "react";

export function MobileHeader() {
  const { t } = useTranslation();
  const params = useParams();
  const [drawerOpen, setDrawerOpen] = useState(false);

  let title = t("app.name");
  if (params.botId) {
    title = t("bots.title");
  } else if (params.projectSlug) {
    title = params.projectSlug;
  } else if (params.orgSlug) {
    title = params.orgSlug;
  }

  return (
    <>
      <header className="h-14 border-b border-border bg-background/80 backdrop-blur-sm flex items-center px-4 gap-3 shrink-0 sticky top-0 z-20">
        <Sheet open={drawerOpen} onOpenChange={setDrawerOpen}>
          <SheetTrigger asChild>
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={t("common.openMenu")}
            >
              <Menu className="size-4" />
            </Button>
          </SheetTrigger>
          <SheetContent
            side="left"
            showCloseButton={false}
            className="w-72 p-0 gap-0"
            aria-describedby={undefined}
          >
            <SheetTitle className="sr-only">{t("common.openMenu")}</SheetTitle>
            <div className="h-14 flex items-center px-4 justify-between">
              <div className="flex items-center gap-2">
                <div className="size-7 rounded-md bg-primary/10 flex items-center justify-center">
                  <Bot className="size-3.5 text-primary" />
                </div>
                <span className="text-sm font-semibold">MaxPanel</span>
              </div>
            </div>
            <Separator />
            <nav className="flex-1 py-2 px-2 space-y-0.5 overflow-y-auto">
              <MobileNavLink
                to="/"
                icon={Building2}
                label={t("orgs.title")}
                onClick={() => setDrawerOpen(false)}
              />
              <MobileNavLink
                to="/invites"
                icon={Inbox}
                label={t("invites.title")}
                onClick={() => setDrawerOpen(false)}
              />
              {params.orgSlug && (
                <>
                  <Separator className="my-1.5" />
                  <MobileNavLink
                    to={`/${params.orgSlug}`}
                    icon={FolderKanban}
                    label={t("projects.title")}
                    onClick={() => setDrawerOpen(false)}
                  />
                  <MobileNavLink
                    to={`/${params.orgSlug}/settings`}
                    icon={Settings}
                    label={t("orgs.settings")}
                    onClick={() => setDrawerOpen(false)}
                  />
                </>
              )}
              {params.orgSlug && params.projectSlug && (
                <>
                  <Separator className="my-1.5" />
                  <MobileNavLink
                    to={`/${params.orgSlug}/${params.projectSlug}`}
                    icon={Bot}
                    label={t("bots.title")}
                    onClick={() => setDrawerOpen(false)}
                  />
                  <MobileNavLink
                    to={`/${params.orgSlug}/${params.projectSlug}/settings`}
                    icon={Settings}
                    label={t("projects.settings")}
                    onClick={() => setDrawerOpen(false)}
                  />
                </>
              )}
              {params.botId && params.orgSlug && params.projectSlug && (
                <>
                  <Separator className="my-1.5" />
                  <MobileNavLink
                    to={`/${params.orgSlug}/${params.projectSlug}/bots/${params.botId}/settings`}
                    icon={Settings}
                    label={t("bots.settings")}
                    onClick={() => setDrawerOpen(false)}
                  />
                </>
              )}
              <Separator className="my-1.5" />
              <MobileNavLink
                to="/settings"
                icon={Settings}
                label={t("settings.title")}
                onClick={() => setDrawerOpen(false)}
              />
            </nav>
          </SheetContent>
        </Sheet>
        <span className="text-sm font-semibold truncate">{title}</span>
      </header>
    </>
  );
}

function MobileNavLink({
  to,
  icon: Icon,
  label,
  onClick,
}: {
  to: string;
  icon: React.ElementType;
  label: string;
  onClick: () => void;
}) {
  const location = useLocation();
  const active = location.pathname === to;

  return (
    <Link
      to={to}
      onClick={onClick}
      className={cn(
        "flex items-center gap-2.5 rounded-md px-2.5 py-2 text-sm transition-colors",
        active
          ? "bg-accent text-accent-foreground font-medium"
          : "text-muted-foreground hover:bg-accent/50 hover:text-foreground",
      )}
    >
      <Icon className="size-4" />
      {label}
    </Link>
  );
}

export function BotBottomTabs() {
  const { t } = useTranslation();
  const location = useLocation();
  const params = useParams();

  if (!params.botId) return null;

  const basePath = `/${params.orgSlug}/${params.projectSlug}/bots/${params.botId}`;

  const tabs = [
    { to: basePath, icon: Bot, label: t("bots.overview"), exact: true },
    {
      to: `${basePath}/chats`,
      icon: MessageSquare,
      label: t("chats.title"),
    },
    {
      to: `${basePath}/events`,
      icon: ScrollText,
      label: t("events.title"),
    },
    {
      to: `${basePath}/console`,
      icon: Terminal,
      label: t("console.title"),
    },
  ];

  return (
    <nav aria-label={t("common.botNavigation")} className="fixed bottom-0 left-0 right-0 h-16 border-t border-border bg-background/95 backdrop-blur-sm flex items-center justify-around z-20 safe-bottom">
      {tabs.map((tab) => {
        const active = tab.exact
          ? location.pathname === tab.to
          : location.pathname.startsWith(tab.to);
        const Icon = tab.icon;

        return (
          <Link
            key={tab.to}
            to={tab.to}
            className={cn(
              "flex flex-col items-center gap-0.5 px-3 py-1.5 text-[10px] transition-colors min-w-[60px]",
              active
                ? "text-primary"
                : "text-muted-foreground active:text-foreground",
            )}
          >
            <Icon className="size-5" />
            <span>{tab.label}</span>
          </Link>
        );
      })}
    </nav>
  );
}
