import api from "@/lib/api-client";
import type {
  ChangePasswordRequest,
  LoginRequest,
  LoginResponse,
  LogoutRequest,
  OkResponse,
  RegisterRequest,
  UpdateMeRequest,
  User,
} from "@/lib/api-types";

export async function login(data: LoginRequest): Promise<LoginResponse> {
  const res = await api.post<LoginResponse>("/auth/login", data);
  return res.data;
}

export async function register(data: RegisterRequest): Promise<LoginResponse> {
  const res = await api.post<LoginResponse>("/auth/register", data);
  return res.data;
}

export async function logout(data: LogoutRequest): Promise<OkResponse> {
  const res = await api.post<OkResponse>("/auth/logout", data);
  return res.data;
}

export async function logoutAll(): Promise<OkResponse> {
  const res = await api.post<OkResponse>("/auth/logout-all");
  return res.data;
}

export async function getMe(): Promise<User> {
  const res = await api.get<User>("/auth/me");
  return res.data;
}

export async function updateMe(data: UpdateMeRequest): Promise<User> {
  const res = await api.patch<User>("/auth/me", data);
  return res.data;
}

export async function changePassword(
  data: ChangePasswordRequest,
): Promise<OkResponse> {
  const res = await api.post<OkResponse>("/auth/change-password", data);
  return res.data;
}
