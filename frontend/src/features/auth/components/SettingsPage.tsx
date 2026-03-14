import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { AxiosError } from "axios";
import { extractApiError } from "@/lib/errors";
import { motion } from "motion/react";
import {
  KeyRound,
  LogOut,
  Moon,
  Monitor,
  Palette,
  Sun,
} from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { PageHeader } from "@/components/layout/PageHeader";
import { getInitials } from "@/lib/user-utils";
import { useThemeStore } from "@/stores/theme";
import { useLocaleStore } from "@/stores/locale";
import { useUser, useLogoutAll } from "../hooks/use-auth";
import * as authApi from "../api";
import { cn } from "@/lib/utils";

export function SettingsPage() {
  const { t } = useTranslation();
  const { data: user } = useUser();
  const { theme, setTheme } = useThemeStore();
  const { locale, setLocale } = useLocaleStore();
  const logoutAll = useLogoutAll();
  const confirm = useConfirm();

  const initials = user?.name ? getInitials(user.name) : "?";

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.15 }}
      className="space-y-8"
    >
      <PageHeader title={t("settings.title")} />

      {/* Profile hero */}
      <div className="flex items-center gap-5">
        <Avatar className="size-16 text-lg">
          <AvatarFallback className="bg-primary/10 text-primary font-semibold">
            {initials}
          </AvatarFallback>
        </Avatar>
        <div className="min-w-0">
          <p className="text-lg font-semibold truncate">
            {user?.name ?? "..."}
          </p>
          <p className="text-sm text-muted-foreground truncate">
            {user?.email}
          </p>
        </div>
      </div>

      <Separator />

      {/* Name */}
      <ProfileSection user={user} />

      <Separator />

      {/* Password */}
      <PasswordSection />

      <Separator />

      {/* Appearance */}
      <section className="space-y-4">
        <div className="flex items-center gap-2">
          <Palette className="size-4 text-muted-foreground" />
          <h2 className="text-sm font-medium">{t("settings.theme")}</h2>
        </div>

        <div className="grid grid-cols-3 gap-2">
          {(
            [
              { value: "light", icon: Sun, label: t("theme.light") },
              { value: "dark", icon: Moon, label: t("theme.dark") },
              { value: "system", icon: Monitor, label: t("theme.system") },
            ] as const
          ).map(({ value, icon: Icon, label }) => (
            <button
              key={value}
              onClick={() => setTheme(value)}
              className={cn(
                "flex flex-col items-center gap-2 rounded-lg border p-4 transition-all",
                theme === value
                  ? "border-primary bg-primary/5 text-foreground"
                  : "border-border hover:border-border hover:bg-muted/50 text-muted-foreground",
              )}
            >
              <Icon className="size-5" />
              <span className="text-xs font-medium">{label}</span>
            </button>
          ))}
        </div>

        <div>
          <h3 className="text-sm font-medium mb-2">{t("settings.language")}</h3>
          <div className="flex gap-2">
            <button
              onClick={() => setLocale("ru")}
              className={cn(
                "flex items-center gap-2 rounded-lg border px-4 py-2.5 text-sm transition-all",
                locale === "ru"
                  ? "border-primary bg-primary/5 font-medium"
                  : "border-border hover:bg-muted/50 text-muted-foreground",
              )}
            >
              🇷🇺 Русский
            </button>
          </div>
          <p className="text-xs text-muted-foreground mt-2">
            {t("common.communityTranslations")}
          </p>
        </div>
      </section>

      <Separator />

      {/* Danger zone */}
      <Button
        variant="destructive"
        size="sm"
        className="gap-1.5"
        onClick={async () => {
          const ok = await confirm({
            description: t("auth.logoutAll"),
            destructive: true,
          });
          if (!ok) return;
          logoutAll.mutate(undefined, {
            onSuccess: () =>
              toast.success(t("auth.allSessionsTerminated")),
          });
        }}
        disabled={logoutAll.isPending}
      >
        <LogOut className="size-3.5" />
        {t("auth.logoutAll")}
      </Button>
    </motion.div>
  );
}

function ProfileSection({
  user,
}: {
  user?: { name: string; email: string };
}) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [name, setName] = useState(user?.name ?? "");
  const [error, setError] = useState<string | null>(null);

  const updateMe = useMutation({
    mutationFn: (data: { name: string }) => authApi.updateMe(data),
    onSuccess: (updated) => {
      queryClient.setQueryData(["auth", "me"], updated);
      toast.success(t("common.save"));
    },
    onError: (err) => {
      setError(extractApiError(err, t("errors.somethingWentWrong")));
    },
  });

  // Sync local state when server data arrives
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    if (user?.name) setName(user.name);
  }, [user?.name]);

  return (
    <section className="space-y-4">
      <h2 className="text-sm font-medium">{t("settings.profile")}</h2>

      <form
        onSubmit={(e) => {
          e.preventDefault();
          setError(null);
          updateMe.mutate({ name });
        }}
        className="space-y-4"
      >
        {error && (
          <div className="rounded-md bg-destructive/10 border border-destructive/20 px-3 py-2 text-sm text-destructive">
            {error}
          </div>
        )}

        <div className="grid gap-4 sm:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="settings-name">{t("auth.name")}</Label>
            <Input
              id="settings-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
              minLength={1}
              maxLength={255}
            />
          </div>
          <div className="space-y-2">
            <Label>{t("auth.email")}</Label>
            <Input
              value={user?.email ?? ""}
              disabled
              className="text-muted-foreground"
            />
          </div>
        </div>

        <Button type="submit" size="sm" disabled={updateMe.isPending}>
          {t("common.save")}
        </Button>
      </form>
    </section>
  );
}

function PasswordSection() {
  const { t } = useTranslation();
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [error, setError] = useState<string | null>(null);

  const changePassword = useMutation({
    mutationFn: (data: { current_password: string; new_password: string }) =>
      authApi.changePassword(data),
    onSuccess: () => {
      toast.success(t("settings.changePassword"));
      setCurrentPassword("");
      setNewPassword("");
    },
    onError: (err) => {
      if (err instanceof AxiosError && err.response?.status === 401) {
        setError(t("auth.invalidCredentials"));
      } else {
        setError(extractApiError(err, t("errors.somethingWentWrong")));
      }
    },
  });

  return (
    <section className="space-y-4">
      <div className="flex items-center gap-2">
        <KeyRound className="size-4 text-muted-foreground" />
        <h2 className="text-sm font-medium">{t("settings.changePassword")}</h2>
      </div>

      <form
        onSubmit={(e) => {
          e.preventDefault();
          setError(null);
          changePassword.mutate({
            current_password: currentPassword,
            new_password: newPassword,
          });
        }}
        className="space-y-4"
      >
        {error && (
          <div className="rounded-md bg-destructive/10 border border-destructive/20 px-3 py-2 text-sm text-destructive">
            {error}
          </div>
        )}

        <div className="grid gap-4 sm:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="current-password">
              {t("auth.currentPassword")}
            </Label>
            <Input
              id="current-password"
              type="password"
              value={currentPassword}
              onChange={(e) => setCurrentPassword(e.target.value)}
              autoComplete="current-password"
              required
              minLength={8}
              maxLength={128}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="new-password">{t("auth.newPassword")}</Label>
            <Input
              id="new-password"
              type="password"
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
              autoComplete="new-password"
              minLength={8}
              maxLength={128}
              required
            />
            <p className="text-xs text-muted-foreground">
              {t("auth.passwordMin")}
            </p>
          </div>
        </div>

        <Button type="submit" size="sm" disabled={changePassword.isPending}>
          {t("settings.changePassword")}
        </Button>
      </form>
    </section>
  );
}
