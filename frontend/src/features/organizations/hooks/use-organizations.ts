import {
  useQuery,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import * as orgApi from "../api";
import type {
  CreateOrgRequest,
  UpdateOrgRequest,
  PaginationParams,
  OrgRole,
} from "@/lib/api-types";

export function useOrganizations(params?: PaginationParams) {
  return useQuery({
    queryKey: ["organizations", params],
    queryFn: () => orgApi.listOrgs(params),
    staleTime: 5 * 60 * 1000,
  });
}

export function useOrganization(slug: string) {
  return useQuery({
    queryKey: ["organizations", slug],
    queryFn: () => orgApi.getOrg(slug),
    enabled: !!slug,
  });
}

export function useCreateOrg() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  return useMutation({
    mutationFn: (data: CreateOrgRequest) => orgApi.createOrg(data),
    onSuccess: (org) => {
      queryClient.invalidateQueries({ queryKey: ["organizations"] });
      navigate(`/${org.slug}`);
    },
  });
}

export function useUpdateOrg(slug: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: UpdateOrgRequest) => orgApi.updateOrg(slug, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["organizations"] });
    },
  });
}

export function useDeleteOrg(slug: string) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  return useMutation({
    mutationFn: () => orgApi.deleteOrg(slug),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["organizations"] });
      queryClient.invalidateQueries({ queryKey: ["projects", slug] });
      queryClient.invalidateQueries({ queryKey: ["bots", slug] });
      navigate("/");
    },
  });
}

export function useOrgMembers(slug: string) {
  return useQuery({
    queryKey: ["organizations", slug, "members"],
    queryFn: () => orgApi.listMembers(slug),
    enabled: !!slug,
  });
}

export function useUpdateMemberRole(slug: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      userId,
      role,
    }: {
      userId: string;
      role: OrgRole;
    }) => orgApi.updateMemberRole(slug, userId, { role }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["organizations", slug, "members"],
      });
    },
  });
}

export function useRemoveMember(slug: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (userId: string) => orgApi.removeMember(slug, userId),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["organizations", slug, "members"],
      });
    },
  });
}

export function useOrgInvites(slug: string) {
  return useQuery({
    queryKey: ["organizations", slug, "invites"],
    queryFn: () => orgApi.listOrgInvites(slug),
    enabled: !!slug,
  });
}

export function useCreateInvite(slug: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: { email: string; role: "admin" | "member" }) =>
      orgApi.createInvite(slug, data),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["organizations", slug, "invites"],
      });
    },
  });
}

export function useRevokeInvite(slug: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (inviteId: string) => orgApi.revokeInvite(slug, inviteId),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["organizations", slug, "invites"],
      });
    },
  });
}

export function useMyInvites() {
  return useQuery({
    queryKey: ["my-invites"],
    queryFn: () => orgApi.listMyInvites(),
  });
}

export function useAcceptInvite() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  return useMutation({
    mutationFn: (token: string) => orgApi.acceptInvite(token),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["organizations"] });
      queryClient.invalidateQueries({ queryKey: ["my-invites"] });
      navigate("/");
    },
  });
}
