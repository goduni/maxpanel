import api from "@/lib/api-client";
import type {
  CreateInviteRequest,
  CreateInviteResponse,
  CreateOrgRequest,
  Invite,
  OkResponse,
  Organization,
  OrganizationMember,
  PaginatedResponse,
  PaginationParams,
  TransferOwnershipRequest,
  UpdateMemberRoleRequest,
  UpdateOrgRequest,
} from "@/lib/api-types";

// Organizations CRUD
export async function listOrgs(
  params?: PaginationParams,
): Promise<PaginatedResponse<Organization>> {
  const res = await api.get<PaginatedResponse<Organization>>(
    "/organizations",
    { params },
  );
  return res.data;
}

export async function getOrg(slug: string): Promise<Organization> {
  const res = await api.get<Organization>(`/organizations/${slug}`);
  return res.data;
}

export async function createOrg(
  data: CreateOrgRequest,
): Promise<Organization> {
  const res = await api.post<Organization>("/organizations", data);
  return res.data;
}

export async function updateOrg(
  slug: string,
  data: UpdateOrgRequest,
): Promise<Organization> {
  const res = await api.patch<Organization>(`/organizations/${slug}`, data);
  return res.data;
}

export async function deleteOrg(slug: string): Promise<OkResponse> {
  const res = await api.delete<OkResponse>(`/organizations/${slug}`);
  return res.data;
}

export async function transferOwnership(
  slug: string,
  data: TransferOwnershipRequest,
): Promise<OkResponse> {
  const res = await api.post<OkResponse>(
    `/organizations/${slug}/transfer-ownership`,
    data,
  );
  return res.data;
}

// Members
export async function listMembers(
  slug: string,
): Promise<{ data: OrganizationMember[] }> {
  const res = await api.get<{ data: OrganizationMember[] }>(
    `/organizations/${slug}/members`,
  );
  return res.data;
}

export async function updateMemberRole(
  slug: string,
  userId: string,
  data: UpdateMemberRoleRequest,
): Promise<OkResponse> {
  const res = await api.patch<OkResponse>(
    `/organizations/${slug}/members/${userId}`,
    data,
  );
  return res.data;
}

export async function removeMember(
  slug: string,
  userId: string,
): Promise<OkResponse> {
  const res = await api.delete<OkResponse>(
    `/organizations/${slug}/members/${userId}`,
  );
  return res.data;
}

// Invites
export async function listOrgInvites(
  slug: string,
): Promise<{ data: Invite[] }> {
  const res = await api.get<{ data: Invite[] }>(
    `/organizations/${slug}/invites`,
  );
  return res.data;
}

export async function createInvite(
  slug: string,
  data: CreateInviteRequest,
): Promise<CreateInviteResponse> {
  const res = await api.post<CreateInviteResponse>(
    `/organizations/${slug}/invites`,
    data,
  );
  return res.data;
}

export async function revokeInvite(
  slug: string,
  inviteId: string,
): Promise<OkResponse> {
  const res = await api.delete<OkResponse>(
    `/organizations/${slug}/invites/${inviteId}`,
  );
  return res.data;
}

// User's pending invites
export async function listMyInvites(): Promise<{ data: Invite[] }> {
  const res = await api.get<{ data: Invite[] }>("/auth/me/invites");
  return res.data;
}

export async function acceptInvite(token: string): Promise<OkResponse> {
  const res = await api.post<OkResponse>(`/invites/${token}/accept`);
  return res.data;
}
