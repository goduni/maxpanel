import type { ReactNode } from "react";
import { Bot } from "lucide-react";

interface EmptyStateProps {
  icon: React.ElementType;
  message: string;
  actions?: ReactNode;
}

export function EmptyState({ icon: Icon, message, actions }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center text-center space-y-4 min-h-[60vh]">
      <div className="space-y-4">
        <div className="size-16 rounded-2xl bg-primary/5 flex items-center justify-center mx-auto">
          <Icon className="size-8 text-primary/40" />
        </div>
        <div className="flex items-center gap-2 justify-center">
          <div className="size-6 rounded-md bg-primary/10 flex items-center justify-center">
            <Bot className="size-3 text-primary" />
          </div>
          <span className="text-sm font-semibold tracking-tight text-muted-foreground/60">
            MaxPanel
          </span>
        </div>
      </div>
      <p className="text-muted-foreground font-medium max-w-xs">{message}</p>
      {actions && <div className="flex gap-2 justify-center">{actions}</div>}
    </div>
  );
}
