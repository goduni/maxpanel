import { useEffect, useRef } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Loader2 } from "lucide-react";
import { toast } from "sonner";
import { useTokenRefresh } from "@/hooks/use-token-refresh";
import { useAcceptInvite } from "../hooks/use-organizations";
import { INVITE_TOKEN_RE } from "@/lib/utils";

export function AcceptInvitePage() {
  const { t } = useTranslation();
  const { token } = useParams<{ token: string }>();
  const navigate = useNavigate();
  const { accessToken, refreshToken } = useTokenRefresh();
  const acceptInvite = useAcceptInvite();
  const acceptedRef = useRef(false);

  // Main logic: validate, redirect guests, accept when ready
  useEffect(() => {
    if (!token || !INVITE_TOKEN_RE.test(token)) {
      navigate("/", { replace: true });
      return;
    }

    if (!accessToken && !refreshToken) {
      sessionStorage.setItem("maxpanel-invite-token", token);
      navigate("/login", { replace: true });
      return;
    }

    // Wait for access token (refresh in progress)
    if (!accessToken) return;

    if (acceptedRef.current) return;
    acceptedRef.current = true;

    acceptInvite.mutate(token, {
      onSuccess: () => {
        toast.success(t("invites.accept"));
      },
      onError: () => {
        toast.error(t("errors.somethingWentWrong"));
        navigate("/", { replace: true });
      },
    });
  }, [token, accessToken, refreshToken]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <div className="min-h-screen bg-background flex items-center justify-center">
      <div className="text-center space-y-3">
        <Loader2 className="size-8 mx-auto animate-spin text-primary" />
        <p className="text-sm text-muted-foreground">{t("common.loading")}</p>
      </div>
    </div>
  );
}
