import { Component, type ErrorInfo, type ReactNode } from "react";
import { ErrorFallback } from "./ErrorFallback";

interface Props {
  children: ReactNode;
  variant?: "page" | "route";
  onReset?: () => void;
}

interface State {
  hasError: boolean;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("ErrorBoundary caught:", error, info);
  }

  render() {
    if (this.state.hasError) {
      const variant = this.props.variant ?? "page";
      return (
        <ErrorFallback
          variant={variant}
          onReset={() => {
            this.props.onReset?.();
            this.setState({ hasError: false });
            if (variant === "page") {
              window.location.href = "/";
            }
          }}
        />
      );
    }

    return this.props.children;
  }
}
