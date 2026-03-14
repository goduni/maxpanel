import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { useAuthStore } from "@/stores/auth";
import * as authApi from "../api";
import type { LoginRequest, RegisterRequest } from "@/lib/api-types";
import { INVITE_TOKEN_RE } from "@/lib/utils";

function getInviteRedirect(): string {
  const pendingInvite = sessionStorage.getItem("maxpanel-invite-token");
  if (pendingInvite && INVITE_TOKEN_RE.test(pendingInvite)) {
    sessionStorage.removeItem("maxpanel-invite-token");
    return `/invite/${encodeURIComponent(pendingInvite)}`;
  }
  sessionStorage.removeItem("maxpanel-invite-token");
  return "/";
}

export function useUser() {
  const { accessToken } = useAuthStore();

  return useQuery({
    queryKey: ["auth", "me"],
    queryFn: authApi.getMe,
    enabled: !!accessToken,
    staleTime: 60 * 1000,
  });
}

export function useLogin() {
  const { setTokens } = useAuthStore();
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  return useMutation({
    mutationFn: (data: LoginRequest) => authApi.login(data),
    onSuccess: (response) => {
      setTokens(response.tokens.access_token, response.tokens.refresh_token);
      queryClient.setQueryData(["auth", "me"], response.user);

      navigate(getInviteRedirect());
    },
  });
}

export function useRegister() {
  const { setTokens } = useAuthStore();
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  return useMutation({
    mutationFn: (data: RegisterRequest) => authApi.register(data),
    onSuccess: (response) => {
      setTokens(response.tokens.access_token, response.tokens.refresh_token);
      queryClient.setQueryData(["auth", "me"], response.user);

      navigate(getInviteRedirect());
    },
  });
}

export function useLogout() {
  const { refreshToken, clearTokens } = useAuthStore();
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  return useMutation({
    mutationFn: () => {
      if (refreshToken) {
        return authApi.logout({ refresh_token: refreshToken });
      }
      return Promise.resolve({ ok: true });
    },
    onSettled: () => {
      clearTokens();
      queryClient.clear();
      navigate("/login");
    },
  });
}

export function useLogoutAll() {
  const { clearTokens } = useAuthStore();
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  return useMutation({
    mutationFn: () => authApi.logoutAll(),
    onSettled: () => {
      clearTokens();
      queryClient.clear();
      navigate("/login");
    },
  });
}
