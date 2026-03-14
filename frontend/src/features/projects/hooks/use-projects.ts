import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import * as projectApi from "../api";
import type { CreateProjectRequest, PaginationParams, ProjectRole } from "@/lib/api-types";

export function useProjects(orgSlug: string, params?: PaginationParams) {
  return useQuery({
    queryKey: ["projects", orgSlug, params],
    queryFn: () => projectApi.listProjects(orgSlug, params),
    enabled: !!orgSlug,
    staleTime: 5 * 60 * 1000,
  });
}

export function useProject(orgSlug: string, projectSlug: string) {
  return useQuery({
    queryKey: ["projects", orgSlug, projectSlug],
    queryFn: () => projectApi.getProject(orgSlug, projectSlug),
    enabled: !!orgSlug && !!projectSlug,
  });
}

export function useCreateProject(orgSlug: string) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  return useMutation({
    mutationFn: (data: CreateProjectRequest) =>
      projectApi.createProject(orgSlug, data),
    onSuccess: (project) => {
      queryClient.invalidateQueries({ queryKey: ["projects", orgSlug] });
      navigate(`/${orgSlug}/${project.slug}`);
    },
  });
}

export function useUpdateProject(orgSlug: string, projectSlug: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: { name: string }) =>
      projectApi.updateProject(orgSlug, projectSlug, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["projects", orgSlug] });
    },
  });
}

export function useDeleteProject(orgSlug: string, projectSlug: string) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  return useMutation({
    mutationFn: () => projectApi.deleteProject(orgSlug, projectSlug),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["projects", orgSlug] });
      queryClient.invalidateQueries({
        queryKey: ["bots", orgSlug, projectSlug],
      });
      navigate(`/${orgSlug}`);
    },
  });
}

export function useProjectMembers(orgSlug: string, projectSlug: string) {
  return useQuery({
    queryKey: ["projects", orgSlug, projectSlug, "members"],
    queryFn: () => projectApi.listProjectMembers(orgSlug, projectSlug),
    enabled: !!orgSlug && !!projectSlug,
  });
}

export function useAddProjectMember(orgSlug: string, projectSlug: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: { user_id: string; role: "admin" | "editor" | "viewer" }) =>
      projectApi.addProjectMember(orgSlug, projectSlug, data),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["projects", orgSlug, projectSlug, "members"],
      });
    },
  });
}

export function useUpdateProjectMemberRole(
  orgSlug: string,
  projectSlug: string,
) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ userId, role }: { userId: string; role: ProjectRole }) =>
      projectApi.updateProjectMemberRole(orgSlug, projectSlug, userId, {
        role,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["projects", orgSlug, projectSlug, "members"],
      });
    },
  });
}

export function useRemoveProjectMember(orgSlug: string, projectSlug: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (userId: string) =>
      projectApi.removeProjectMember(orgSlug, projectSlug, userId),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["projects", orgSlug, projectSlug, "members"],
      });
    },
  });
}
