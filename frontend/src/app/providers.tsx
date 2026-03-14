import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { type ReactNode, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { MotionConfig } from "motion/react";
import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { ConfirmProvider } from "@/components/ui/confirm-dialog";

// Import to trigger side-effects on load
import "@/stores/theme";
import "@/i18n";

// eslint-disable-next-line react-refresh/only-export-components -- queryClient must be shared with routes.tsx for error boundary retry
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30 * 1000,
      refetchOnWindowFocus: true,
      retry: 1,
    },
  },
});

export function Providers({ children }: { children: ReactNode }) {
  const { t } = useTranslation();

  // Listen for rate limit events from Axios interceptor
  useEffect(() => {
    const handler = (e: Event) => {
      const { retryAfter } = (e as CustomEvent).detail;
      toast.warning(t("errors.tooManyRequests", { seconds: retryAfter }));
    };
    window.addEventListener("maxpanel:rate-limited", handler);
    return () => window.removeEventListener("maxpanel:rate-limited", handler);
  }, [t]);

  return (
    <MotionConfig reducedMotion="user">
      <QueryClientProvider client={queryClient}>
        <TooltipProvider>
          <ConfirmProvider>
            {children}
            <Toaster position="top-right" richColors />
          </ConfirmProvider>
        </TooltipProvider>
      </QueryClientProvider>
    </MotionConfig>
  );
}
