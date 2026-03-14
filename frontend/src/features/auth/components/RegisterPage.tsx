import { useState } from "react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { motion } from "motion/react";
import { AxiosError } from "axios";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useRegister } from "../hooks/use-auth";
import { extractApiError } from "@/lib/errors";
import type { ApiErrorResponse } from "@/lib/api-types";
import { AuthShell } from "./AuthShell";

export function RegisterPage() {
  const { t } = useTranslation();
  const register = useRegister();
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setFieldErrors({});

    register.mutate(
      { name, email, password },
      {
        onError: (err) => {
          if (err instanceof AxiosError) {
            const data = err.response?.data as ApiErrorResponse | undefined;
            if (err.response?.status === 409) {
              setError(t("auth.emailTaken"));
            } else if (data?.error?.details) {
              const errors: Record<string, string> = {};
              for (const d of data.error.details) {
                errors[d.field] = d.message;
              }
              setFieldErrors(errors);
            } else {
              setError(extractApiError(err, t("errors.somethingWentWrong")));
            }
          } else {
            setError(extractApiError(err, t("errors.somethingWentWrong")));
          }
        },
      },
    );
  };

  return (
    <AuthShell>
      <motion.div
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.4, ease: [0.25, 0.46, 0.45, 0.94] }}
        className="w-full"
      >
        <div className="mb-8">
          <h1 className="text-2xl font-semibold tracking-tight">
            {t("auth.register")}
          </h1>
          <p className="text-sm text-muted-foreground mt-1.5">
            {t("app.description")}
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          {error && (
            <motion.div
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: "auto" }}
              className="rounded-md bg-destructive/10 border border-destructive/20 px-3 py-2.5 text-sm text-destructive"
            >
              {error}
            </motion.div>
          )}

          <div className="space-y-2">
            <Label htmlFor="name">{t("auth.name")}</Label>
            <Input
              id="name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              autoComplete="name"
              autoFocus
              required
              minLength={1}
              maxLength={255}
            />
            {fieldErrors.name && (
              <p className="text-xs text-destructive">{fieldErrors.name}</p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="email">{t("auth.email")}</Label>
            <Input
              id="email"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="you@example.com"
              autoComplete="email"
              required
              maxLength={254}
            />
            {fieldErrors.email && (
              <p className="text-xs text-destructive">{fieldErrors.email}</p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="password">{t("auth.password")}</Label>
            <Input
              id="password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="••••••••"
              autoComplete="new-password"
              minLength={8}
              maxLength={128}
              required
            />
            <p className="text-xs text-muted-foreground">
              {t("auth.passwordMin")}
            </p>
            {fieldErrors.password && (
              <p className="text-xs text-destructive">
                {fieldErrors.password}
              </p>
            )}
          </div>

          <Button
            type="submit"
            className="w-full"
            disabled={register.isPending}
          >
            {register.isPending ? t("common.loading") : t("auth.register")}
          </Button>
        </form>

        <p className="mt-6 text-center text-sm text-muted-foreground">
          {t("auth.hasAccount")}{" "}
          <Link
            to="/login"
            className="text-primary underline-offset-4 hover:underline font-medium"
          >
            {t("auth.login")}
          </Link>
        </p>
      </motion.div>
    </AuthShell>
  );
}
