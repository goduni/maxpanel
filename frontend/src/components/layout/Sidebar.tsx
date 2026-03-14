import { useEffect } from "react";
import { Link, useLocation, useNavigate, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { motion } from "motion/react";
import {
  Bot,
  Building2,
  ChevronDown,
  ChevronLeft,
  FolderKanban,
  Github,
  Inbox,
  LogOut,
  MessageSquare,
  ScrollText,
  Settings,
  Terminal,
  User,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { getInitials } from "@/lib/user-utils";
import { useSidebarStore } from "@/stores/sidebar";
import { useLogout, useUser } from "@/features/auth/hooks/use-auth";
import { useOrganizations } from "@/features/organizations/hooks/use-organizations";
import { useProjects } from "@/features/projects/hooks/use-projects";
import { useBots } from "@/features/bots/hooks/use-bots";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Separator } from "@/components/ui/separator";

interface NavItem {
  to: string;
  icon: React.ElementType;
  label: string;
  exact?: boolean;
}

export function Sidebar() {
  const { t } = useTranslation();
  const location = useLocation();
  const navigate = useNavigate();
  const params = useParams();
  const { collapsed, toggle } = useSidebarStore();
  const { data: user } = useUser();
  const logout = useLogout();
  const { data: orgsData } = useOrganizations();

  const orgSlug = params.orgSlug;
  const projectSlug = params.projectSlug;
  const botId = params.botId;

  const { data: projectsData } = useProjects(orgSlug ?? "", undefined);
  const { data: botsData } = useBots(orgSlug ?? "", projectSlug ?? "");

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "b") {
        e.preventDefault();
        toggle();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [toggle]);

  // Context-dependent navigation
  const navItems: NavItem[] = [];

  if (!orgSlug) {
    // Root level — show orgs and invites
    navItems.push(
      { to: "/", icon: Building2, label: t("orgs.title"), exact: true },
      { to: "/invites", icon: Inbox, label: t("invites.title") },
    );
  } else if (!projectSlug) {
    // Inside org — show projects + org settings
    navItems.push(
      { to: `/${orgSlug}`, icon: FolderKanban, label: t("projects.title"), exact: true },
      { to: `/${orgSlug}/settings`, icon: Settings, label: t("orgs.settings") },
    );
  } else if (!botId) {
    // Inside project — show bots + project settings
    navItems.push(
      { to: `/${orgSlug}/${projectSlug}`, icon: Bot, label: t("bots.title"), exact: true },
      { to: `/${orgSlug}/${projectSlug}/settings`, icon: Settings, label: t("projects.settings") },
    );
  } else {
    // Inside bot — show bot sections
    const base = `/${orgSlug}/${projectSlug}/bots/${botId}`;
    navItems.push(
      { to: base, icon: Bot, label: t("bots.overview"), exact: true },
      { to: `${base}/chats`, icon: MessageSquare, label: t("chats.title") },
      { to: `${base}/events`, icon: ScrollText, label: t("events.title") },
      { to: `${base}/console`, icon: Terminal, label: t("console.title") },
      { to: `${base}/settings`, icon: Settings, label: t("bots.settings") },
    );
  }

  const isActive = (item: NavItem) => {
    if (item.exact) return location.pathname === item.to;
    return location.pathname.startsWith(item.to);
  };

  const initials = user?.name ? getInitials(user.name) : "?";

  const currentOrg = orgsData?.data.find((o) => o.slug === orgSlug);
  const currentProject = projectsData?.data.find((p) => p.slug === projectSlug);
  const currentBot = botsData?.data.find((b) => b.id === botId);

  return (
    <motion.aside
      animate={{ width: collapsed ? 64 : 240 }}
      transition={{ type: "spring", stiffness: 300, damping: 30 }}
      className="h-screen sticky top-0 border-r border-sidebar-border bg-sidebar-background flex flex-col shrink-0 overflow-hidden"
    >
      {/* Header — context switchers */}
      <div className="shrink-0">
        <div className="h-14 flex items-center px-3 gap-2">
          {collapsed ? (
            <Tooltip>
              <TooltipTrigger asChild>
                <button
                  onClick={toggle}
                  className="mx-auto flex items-center justify-center size-8 rounded-md bg-primary/10 hover:bg-primary/15 transition-colors"
                >
                  <Bot className="size-4 text-primary" />
                </button>
              </TooltipTrigger>
              <TooltipContent side="right">{t("common.expand")}</TooltipContent>
            </Tooltip>
          ) : (
            <>
              {/* Org switcher or brand */}
              {orgSlug && orgsData ? (
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <button className="flex items-center gap-2 min-w-0 hover:bg-sidebar-accent/50 rounded-md px-1.5 py-1 transition-colors">
                      <div className="size-7 rounded-md bg-primary/10 flex items-center justify-center shrink-0">
                        <Building2 className="size-3.5 text-primary" />
                      </div>
                      <span className="text-sm font-semibold tracking-tight truncate">
                        {currentOrg?.name ?? orgSlug}
                      </span>
                      <ChevronDown className="size-3 text-muted-foreground shrink-0" />
                    </button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="start" className="w-52">
                    {orgsData.data.map((org) => (
                      <DropdownMenuItem
                        key={org.id}
                        onClick={() => navigate(`/${org.slug}`)}
                        className={cn(
                          "gap-2",
                          org.slug === orgSlug && "bg-accent font-medium",
                        )}
                      >
                        <Building2 className="size-3.5 shrink-0" />
                        {org.name}
                      </DropdownMenuItem>
                    ))}
                    <DropdownMenuSeparator />
                    <DropdownMenuItem onClick={() => navigate("/")} className="gap-2 text-muted-foreground">
                      <Building2 className="size-3.5" />
                      {t("orgs.title")}
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              ) : (
                <Link to="/" className="flex items-center gap-2.5 min-w-0">
                  <div className="size-7 rounded-md bg-primary/10 flex items-center justify-center shrink-0">
                    <Bot className="size-3.5 text-primary" />
                  </div>
                  <span className="text-sm font-semibold tracking-tight truncate">
                    MaxPanel
                  </span>
                </Link>
              )}
              <Button
                variant="ghost"
                size="icon-xs"
                onClick={toggle}
                className="ml-auto shrink-0 text-muted-foreground"
                aria-label={t("common.collapse")}
              >
                <ChevronLeft className="size-3.5" />
              </Button>
            </>
          )}
        </div>

        {/* Project switcher — only when inside an org */}
        {!collapsed && orgSlug && projectsData && (
          <div className="px-3 pb-1">
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button className="w-full flex items-center gap-2 hover:bg-sidebar-accent/50 rounded-md px-1.5 py-1 transition-colors text-left">
                  <FolderKanban className="size-3.5 text-muted-foreground shrink-0" />
                  <span className="text-xs truncate text-sidebar-foreground">
                    {currentProject?.name ?? (projectSlug ? projectSlug : t("projects.title"))}
                  </span>
                  <ChevronDown className="size-3 text-muted-foreground shrink-0 ml-auto" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start" className="w-52">
                {projectsData.data.map((proj) => (
                  <DropdownMenuItem
                    key={proj.id}
                    onClick={() => navigate(`/${orgSlug}/${proj.slug}`)}
                    className={cn(
                      "gap-2",
                      proj.slug === projectSlug && "bg-accent font-medium",
                    )}
                  >
                    <FolderKanban className="size-3.5 shrink-0" />
                    {proj.name}
                  </DropdownMenuItem>
                ))}
                {projectsData.data.length === 0 && (
                  <div className="px-2 py-3 text-xs text-muted-foreground text-center">
                    {t("projects.emptyState")}
                  </div>
                )}
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        )}

        {/* Bot switcher — when inside a project */}
        {!collapsed && projectSlug && botsData && botsData.data.length > 0 && (
          <div className="px-3 pb-2">
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button className="w-full flex items-center gap-2 hover:bg-sidebar-accent/50 rounded-md px-1.5 py-1 transition-colors text-left">
                  <div className="relative shrink-0">
                    <Bot className="size-3.5 text-muted-foreground" />
                    {currentBot && (
                      <span className={cn(
                        "absolute -top-0.5 -right-0.5 size-1.5 rounded-full",
                        currentBot.is_active ? "bg-emerald-500" : "bg-muted-foreground/40",
                      )} />
                    )}
                  </div>
                  <span className="text-xs truncate text-sidebar-foreground">
                    {currentBot?.name ?? t("bots.title")}
                  </span>
                  <ChevronDown className="size-3 text-muted-foreground shrink-0 ml-auto" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start" className="w-52">
                {botsData.data.map((bot) => (
                  <DropdownMenuItem
                    key={bot.id}
                    onClick={() => navigate(`/${orgSlug}/${projectSlug}/bots/${bot.id}`)}
                    className={cn(
                      "gap-2",
                      bot.id === botId && "bg-accent font-medium",
                    )}
                  >
                    <div className="relative shrink-0">
                      <Bot className="size-3.5" />
                      <span className={cn(
                        "absolute -top-0.5 -right-0.5 size-1.5 rounded-full",
                        bot.is_active ? "bg-emerald-500" : "bg-muted-foreground/40",
                      )} />
                    </div>
                    {bot.name}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        )}

        <Separator />
      </div>

      {/* Context navigation */}
      <nav className="flex-1 py-2 px-2 space-y-0.5 overflow-y-auto flex flex-col">
        {navItems.map((item) => (
          <SidebarItem
            key={item.to}
            item={item}
            active={isActive(item)}
            collapsed={collapsed}
            orgSlug={orgSlug}
          />
        ))}

        {/* Author credit — pinned to bottom of nav */}
        <div className="mt-auto" />
        {collapsed ? (
          <div className="flex flex-col items-center gap-0.5 pt-1">
            <Tooltip>
              <TooltipTrigger asChild>
                <a
                  href="https://t.me/goduniblog"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center justify-center size-8 rounded-md text-muted-foreground/60 hover:text-[oklch(0.55_0.15_250)] hover:bg-sidebar-accent/50 transition-colors"
                >
                  <TelegramIcon className="size-4" />
                </a>
              </TooltipTrigger>
              <TooltipContent side="right">Telegram — @goduniblog</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <a
                  href="https://github.com/goduni"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center justify-center size-8 rounded-md text-muted-foreground/60 hover:text-muted-foreground hover:bg-sidebar-accent/50 transition-colors"
                >
                  <Github className="size-4" />
                </a>
              </TooltipTrigger>
              <TooltipContent side="right">GitHub — goduni</TooltipContent>
            </Tooltip>
          </div>
        ) : (
          <div className="mx-1 mt-1 rounded-lg border border-sidebar-border/60 bg-sidebar-accent/30 px-3 py-2.5 flex items-center gap-3">
            <span className="text-xs text-muted-foreground font-medium tracking-wide">by Юни</span>
            <div className="flex items-center gap-1 ml-auto">
              <a
                href="https://t.me/goduniblog"
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center justify-center size-7 rounded-md text-muted-foreground/60 hover:text-[oklch(0.55_0.15_250)] hover:bg-sidebar-accent transition-colors"
                aria-label="Telegram"
              >
                <TelegramIcon className="size-4" />
              </a>
              <a
                href="https://github.com/goduni"
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center justify-center size-7 rounded-md text-muted-foreground/60 hover:text-muted-foreground hover:bg-sidebar-accent transition-colors"
                aria-label="GitHub"
              >
                <Github className="size-4" />
              </a>
            </div>
          </div>
        )}
      </nav>

      <Separator />

      {/* User menu */}
      <div className="p-2 shrink-0">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <button
              className={cn(
                "w-full flex items-center gap-2.5 rounded-md px-2 py-1.5 text-sm",
                "hover:bg-sidebar-accent transition-colors",
                collapsed && "justify-center",
              )}
            >
              <Avatar className="size-7">
                <AvatarFallback className="text-xs bg-primary/10 text-primary">
                  {initials}
                </AvatarFallback>
              </Avatar>
              {!collapsed && (
                <span className="truncate text-sidebar-foreground">
                  {user?.name ?? "..."}
                </span>
              )}
            </button>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            side={collapsed ? "right" : "top"}
            align="start"
            className="w-48"
          >
            <DropdownMenuItem asChild>
              <Link to="/settings" className="flex items-center gap-2">
                <User className="size-4" />
                {t("settings.profile")}
              </Link>
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              onClick={() => logout.mutate()}
              className="flex items-center gap-2 text-destructive focus:text-destructive"
            >
              <LogOut className="size-4" />
              {t("auth.logout")}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </motion.aside>
  );
}

function TelegramIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" className={className}>
      <path d="M11.944 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.056 0zm4.962 7.224c.1-.002.321.023.465.14a.506.506 0 0 1 .171.325c.016.093.036.306.02.472-.18 1.898-.962 6.502-1.36 8.627-.168.9-.499 1.201-.82 1.23-.696.065-1.225-.46-1.9-.902-1.056-.693-1.653-1.124-2.678-1.8-1.185-.78-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.14-5.061 3.345-.48.33-.913.49-1.302.48-.428-.008-1.252-.241-1.865-.44-.752-.245-1.349-.374-1.297-.789.027-.216.325-.437.893-.663 3.498-1.524 5.83-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635z" />
    </svg>
  );
}

function SidebarItem({
  item,
  active,
  collapsed,
  orgSlug,
}: {
  item: NavItem;
  active: boolean;
  collapsed: boolean;
  orgSlug?: string;
}) {
  const Icon = item.icon;

  const content = (
    <Link
      to={item.to}
      className={cn(
        "relative flex items-center gap-2.5 rounded-md px-2 py-1.5 text-sm transition-colors",
        active
          ? "bg-sidebar-accent text-sidebar-accent-foreground font-medium"
          : "text-sidebar-foreground hover:bg-sidebar-accent/50",
        collapsed && "justify-center px-0",
      )}
    >
      {active && (
        <motion.div
          layoutId={`sidebar-indicator-${orgSlug ?? "root"}`}
          className="absolute left-0 top-1 bottom-1 w-0.5 rounded-full bg-primary shadow-[0_0_8px_var(--primary)]"
          transition={{ type: "spring", stiffness: 300, damping: 30 }}
        />
      )}
      <Icon className="size-4 shrink-0" />
      {!collapsed && <span className="truncate">{item.label}</span>}
    </Link>
  );

  if (collapsed) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>{content}</TooltipTrigger>
        <TooltipContent side="right">{item.label}</TooltipContent>
      </Tooltip>
    );
  }

  return content;
}
