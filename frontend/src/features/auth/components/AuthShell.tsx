import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Bot } from "lucide-react";

export function AuthShell({ children }: { children: ReactNode }) {
  const { t } = useTranslation();

  return (
    <div className="min-h-screen bg-background flex">
      {/* Left decorative panel — hidden on mobile */}
      <div className="hidden lg:flex lg:w-1/2 relative overflow-hidden bg-card border-r border-border items-center justify-center">
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top_left,var(--primary)_0%,transparent_50%)] opacity-[0.07]" />
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_bottom_right,var(--primary)_0%,transparent_50%)] opacity-[0.05]" />
        <div className="relative z-10 px-12 max-w-lg">
          <div className="flex items-center gap-3 mb-6">
            <div className="size-10 rounded-lg bg-primary/10 flex items-center justify-center">
              <Bot className="size-5 text-primary" />
            </div>
            <span className="text-xl font-semibold tracking-tight">
              {t("app.name")}
            </span>
          </div>
          <h2 className="text-3xl font-bold tracking-tight leading-tight mb-3">
            {t("landing.headline")}
            <br />
            <span className="text-primary">{t("landing.headlineAccent")}</span>
          </h2>
          <p className="text-muted-foreground leading-relaxed">
            {t("landing.description")}
          </p>
        </div>
        {/* Subtle grid pattern */}
        <div
          className="absolute inset-0 opacity-[0.04]"
          style={{
            backgroundImage:
              "linear-gradient(var(--foreground) 1px, transparent 1px), linear-gradient(90deg, var(--foreground) 1px, transparent 1px)",
            backgroundSize: "40px 40px",
          }}
        />
        <div className="absolute inset-0 opacity-[0.15] mix-blend-overlay"
          style={{ backgroundImage: "url(\"data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)'/%3E%3C/svg%3E\")" }}
        />
      </div>

      {/* Right form panel */}
      <div className="flex-1 flex items-center justify-center p-6 sm:p-8">
        <div className="w-full max-w-sm">
          {/* Mobile logo */}
          <div className="flex items-center gap-2.5 mb-8 lg:hidden">
            <div className="size-8 rounded-md bg-primary/10 flex items-center justify-center">
              <Bot className="size-4 text-primary" />
            </div>
            <span className="text-lg font-semibold tracking-tight">
              {t("app.name")}
            </span>
          </div>
          {children}
        </div>
      </div>
    </div>
  );
}
