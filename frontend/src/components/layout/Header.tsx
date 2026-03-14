import { useLocation, useParams, Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useIsFetching } from "@tanstack/react-query";
import {
  ChevronRight,
  Moon,
  Sun,
  Monitor,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useThemeStore } from "@/stores/theme";
import { cn } from "@/lib/utils";
import { useOrganization } from "@/features/organizations/hooks/use-organizations";
import { useProject } from "@/features/projects/hooks/use-projects";
import { useBot } from "@/features/bots/hooks/use-bots";

export function Header() {
  const { t } = useTranslation();
  const { theme, setTheme } = useThemeStore();
  const isFetching = useIsFetching();
  const params = useParams();
  const location = useLocation();

  const { data: org } = useOrganization(params.orgSlug ?? "");
  const { data: project } = useProject(
    params.orgSlug ?? "",
    params.projectSlug ?? "",
  );
  const { data: bot } = useBot(
    params.orgSlug ?? "",
    params.projectSlug ?? "",
    params.botId ?? "",
  );

  // Build breadcrumbs from URL
  const crumbs: { label: string; to: string }[] = [];

  if (params.orgSlug) {
    crumbs.push({ label: org?.name ?? params.orgSlug, to: `/${params.orgSlug}` });
  }
  if (params.projectSlug) {
    crumbs.push({
      label: project?.name ?? params.projectSlug,
      to: `/${params.orgSlug}/${params.projectSlug}`,
    });
  }
  if (params.botId) {
    crumbs.push({
      label: bot?.name ?? params.botId.slice(0, 8),
      to: `/${params.orgSlug}/${params.projectSlug}/bots/${params.botId}`,
    });

    // Sub-routes
    const subPath = location.pathname.split("/").pop();
    if (
      subPath &&
      ["chats", "events", "console", "settings"].includes(subPath)
    ) {
      const labelMap: Record<string, string> = {
        chats: t("chats.title"),
        events: t("events.title"),
        console: t("console.title"),
        settings: t("bots.settings"),
      };
      crumbs.push({
        label: labelMap[subPath] ?? subPath,
        to: location.pathname,
      });
    }
  }

  const themeIcon = theme === "dark" ? Moon : theme === "light" ? Sun : Monitor;
  const ThemeIcon = themeIcon;

  return (
    <header className="h-14 border-b border-border bg-background/80 backdrop-blur-sm flex items-center px-4 gap-4 shrink-0 sticky top-0 z-10 relative">
      {/* Breadcrumbs */}
      <nav className="flex items-center gap-1 text-sm min-w-0 flex-1">
        <Link
          to="/"
          className="text-muted-foreground hover:text-foreground transition-colors shrink-0"
        >
          {t("orgs.title")}
        </Link>
        {crumbs.map((crumb, i) => (
          <span key={crumb.to} className="flex items-center gap-1 min-w-0">
            <ChevronRight className="size-3.5 text-muted-foreground shrink-0" />
            <Link
              to={crumb.to}
              className={cn(
                "truncate transition-colors",
                i === crumbs.length - 1
                  ? "text-foreground font-medium"
                  : "text-muted-foreground hover:text-foreground",
              )}
            >
              {crumb.label}
            </Link>
          </span>
        ))}
      </nav>

      {isFetching > 0 && (
        <div className="absolute top-0 left-0 right-0 h-0.5 bg-primary/60 animate-pulse" />
      )}

      {/* Theme toggle */}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon-sm" aria-label={t("settings.theme")}>
            <ThemeIcon className="size-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem onClick={() => setTheme("light")}>
            <Sun className="size-4 mr-2" />
            {t("theme.light")}
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => setTheme("dark")}>
            <Moon className="size-4 mr-2" />
            {t("theme.dark")}
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => setTheme("system")}>
            <Monitor className="size-4 mr-2" />
            {t("theme.system")}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </header>
  );
}
