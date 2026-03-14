import api from "@/lib/api-client";
import type {
  AddProjectMemberRequest,
  CreateProjectRequest,
  OkResponse,
  PaginatedResponse,
  PaginationParams,
  Project,
  ProjectMember,
  UpdateMemberRoleRequest,
  UpdateProjectRequest,
} from "@/lib/api-types";

export async function listProjects(
  orgSlug: string,
  params?: PaginationParams,
): Promise<PaginatedResponse<Project>> {
  const res = await api.get<PaginatedResponse<Project>>(
    `/organizations/${orgSlug}/projects`,
    { params },
  );
  return res.data;
}

export async function getProject(
  orgSlug: string,
  projectSlug: string,
): Promise<Project> {
  const res = await api.get<Project>(
    `/organizations/${orgSlug}/projects/${projectSlug}`,
  );
  return res.data;
}

export async function createProject(
  orgSlug: string,
  data: CreateProjectRequest,
): Promise<Project> {
  const res = await api.post<Project>(
    `/organizations/${orgSlug}/projects`,
    data,
  );
  return res.data;
}

export async function updateProject(
  orgSlug: string,
  projectSlug: string,
  data: UpdateProjectRequest,
): Promise<Project> {
  const res = await api.patch<Project>(
    `/organizations/${orgSlug}/projects/${projectSlug}`,
    data,
  );
  return res.data;
}

export async function deleteProject(
  orgSlug: string,
  projectSlug: string,
): Promise<OkResponse> {
  const res = await api.delete<OkResponse>(
    `/organizations/${orgSlug}/projects/${projectSlug}`,
  );
  return res.data;
}

// Members
export async function listProjectMembers(
  orgSlug: string,
  projectSlug: string,
): Promise<{ data: ProjectMember[] }> {
  const res = await api.get<{ data: ProjectMember[] }>(
    `/organizations/${orgSlug}/projects/${projectSlug}/members`,
  );
  return res.data;
}

export async function addProjectMember(
  orgSlug: string,
  projectSlug: string,
  data: AddProjectMemberRequest,
): Promise<OkResponse> {
  const res = await api.post<OkResponse>(
    `/organizations/${orgSlug}/projects/${projectSlug}/members`,
    data,
  );
  return res.data;
}

export async function updateProjectMemberRole(
  orgSlug: string,
  projectSlug: string,
  userId: string,
  data: UpdateMemberRoleRequest,
): Promise<OkResponse> {
  const res = await api.patch<OkResponse>(
    `/organizations/${orgSlug}/projects/${projectSlug}/members/${userId}`,
    data,
  );
  return res.data;
}

export async function removeProjectMember(
  orgSlug: string,
  projectSlug: string,
  userId: string,
): Promise<OkResponse> {
  const res = await api.delete<OkResponse>(
    `/organizations/${orgSlug}/projects/${projectSlug}/members/${userId}`,
  );
  return res.data;
}
