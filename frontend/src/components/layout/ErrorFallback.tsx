import { AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";
import i18n from "@/i18n";

interface ErrorFallbackProps {
  variant: "page" | "route";
  onReset: () => void;
}

export function ErrorFallback({ variant, onReset }: ErrorFallbackProps) {
  if (variant === "page") {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="text-center space-y-4">
          <AlertTriangle className="size-10 mx-auto text-destructive/60" />
          <p className="text-lg font-medium text-foreground">
            {i18n.t("errors.somethingWentWrong")}
          </p>
          <Button variant="outline" onClick={onReset}>
            {i18n.t("common.back")}
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center justify-center gap-4 p-10">
      <AlertTriangle className="size-8 text-destructive/60" />
      <p className="text-sm font-medium text-foreground">
        {i18n.t("errors.somethingWentWrong")}
      </p>
      <Button variant="outline" size="sm" onClick={onReset}>
        {i18n.t("common.retry")}
      </Button>
    </div>
  );
}
