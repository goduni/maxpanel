import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import * as botApi from "../api";
import type {
  Bot,
  CreateApiKeyRequest,
  CreateBotRequest,
  PaginatedResponse,
  PaginationParams,
} from "@/lib/api-types";

function updateBotInList(
  queryClient: ReturnType<typeof useQueryClient>,
  orgSlug: string,
  projectSlug: string,
  botId: string,
  updater: (bot: Bot) => Bot,
) {
  // Update detail cache
  const detailKey = ["bots", orgSlug, projectSlug, botId];
  queryClient.setQueryData(detailKey, (old: Bot | undefined) =>
    old ? updater(old) : old,
  );

  // Update list cache (all paginated variants)
  queryClient.setQueriesData<PaginatedResponse<Bot>>(
    { queryKey: ["bots", orgSlug, projectSlug] },
    (old) => {
      if (!old) return old;
      return {
        ...old,
        data: old.data.map((b) => (b.id === botId ? updater(b) : b)),
      };
    },
  );
}

export function useBots(
  orgSlug: string,
  projectSlug: string,
  params?: PaginationParams,
) {
  return useQuery({
    queryKey: ["bots", orgSlug, projectSlug, params],
    queryFn: () => botApi.listBots(orgSlug, projectSlug, params),
    enabled: !!orgSlug && !!projectSlug,
    staleTime: 30 * 1000,
  });
}

export function useBot(
  orgSlug: string,
  projectSlug: string,
  botId: string,
) {
  return useQuery({
    queryKey: ["bots", orgSlug, projectSlug, botId],
    queryFn: () => botApi.getBot(orgSlug, projectSlug, botId),
    enabled: !!orgSlug && !!projectSlug && !!botId,
  });
}

export function useCreateBot(orgSlug: string, projectSlug: string) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  return useMutation({
    mutationFn: (data: CreateBotRequest) =>
      botApi.createBot(orgSlug, projectSlug, data),
    onSuccess: (bot) => {
      queryClient.invalidateQueries({
        queryKey: ["bots", orgSlug, projectSlug],
      });
      navigate(`/${orgSlug}/${projectSlug}/bots/${bot.id}`);
    },
  });
}

export function useUpdateBot(
  orgSlug: string,
  projectSlug: string,
  botId: string,
) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: { name?: string; history_limit?: number }) =>
      botApi.updateBot(orgSlug, projectSlug, botId, data),
    onSuccess: (updatedBot) => {
      updateBotInList(queryClient, orgSlug, projectSlug, botId, () => updatedBot);
    },
  });
}

export function useDeleteBot(
  orgSlug: string,
  projectSlug: string,
  botId: string,
) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  return useMutation({
    mutationFn: () => botApi.deleteBot(orgSlug, projectSlug, botId),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["bots", orgSlug, projectSlug],
      });
      navigate(`/${orgSlug}/${projectSlug}`);
    },
  });
}

export function useStartBot(
  orgSlug: string,
  projectSlug: string,
  botId: string,
) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => botApi.startBot(botId),
    onMutate: async () => {
      await queryClient.cancelQueries({
        queryKey: ["bots", orgSlug, projectSlug],
      });
      const previousDetail = queryClient.getQueryData<Bot>([
        "bots", orgSlug, projectSlug, botId,
      ]);
      updateBotInList(queryClient, orgSlug, projectSlug, botId, (b) => ({
        ...b,
        is_active: true,
      }));
      return { previousDetail };
    },
    onError: (_err, _vars, context) => {
      if (context?.previousDetail) {
        updateBotInList(queryClient, orgSlug, projectSlug, botId, () =>
          context.previousDetail!,
        );
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({
        queryKey: ["bots", orgSlug, projectSlug],
      });
    },
  });
}

export function useStopBot(
  orgSlug: string,
  projectSlug: string,
  botId: string,
) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => botApi.stopBot(botId),
    onMutate: async () => {
      await queryClient.cancelQueries({
        queryKey: ["bots", orgSlug, projectSlug],
      });
      const previousDetail = queryClient.getQueryData<Bot>([
        "bots", orgSlug, projectSlug, botId,
      ]);
      updateBotInList(queryClient, orgSlug, projectSlug, botId, (b) => ({
        ...b,
        is_active: false,
      }));
      return { previousDetail };
    },
    onError: (_err, _vars, context) => {
      if (context?.previousDetail) {
        updateBotInList(queryClient, orgSlug, projectSlug, botId, () =>
          context.previousDetail!,
        );
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({
        queryKey: ["bots", orgSlug, projectSlug],
      });
    },
  });
}

export function useVerifyBot(
  orgSlug: string,
  projectSlug: string,
  botId: string,
) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => botApi.verifyBot(botId),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["bots", orgSlug, projectSlug],
      });
    },
  });
}

// API Keys

export function useApiKeys(botId: string) {
  return useQuery({
    queryKey: ["api-keys", botId],
    queryFn: () => botApi.listApiKeys(botId),
    enabled: !!botId,
  });
}

export function useCreateApiKey(botId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: CreateApiKeyRequest) =>
      botApi.createApiKey(botId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["api-keys", botId] });
    },
  });
}

export function useDeleteApiKey(botId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (keyId: string) => botApi.deleteApiKey(botId, keyId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["api-keys", botId] });
    },
  });
}
