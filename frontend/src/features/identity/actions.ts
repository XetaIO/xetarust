"use server";

import { revalidatePath } from "next/cache";
import { redirect } from "next/navigation";

import { apiFetch } from "@/lib/api/client";
import { field, type FormState, toFormState } from "@/lib/forms";
import type { AuthResponse } from "@/types/api/identity/AuthResponse";
import type { ChangeRoleRequest } from "@/types/api/identity/ChangeRoleRequest";
import type { LoginRequest } from "@/types/api/identity/LoginRequest";
import type { RegisterRequest } from "@/types/api/identity/RegisterRequest";
import type { RoleDto } from "@/types/api/identity/RoleDto";
import type { UserDto } from "@/types/api/identity/UserDto";

import { safeRedirectTarget } from "./redirect";
import { clearSession, storeSession } from "./session";

// ------------------------------------------------------------ Authentication

/** Logs the user in against the Rust API and stores the JWT in the session cookie. */
export async function login(_: FormState, data: FormData): Promise<FormState> {
  const body: LoginRequest = { email: field(data, "email"), password: field(data, "password") };

  try {
    await storeSession(await apiFetch<AuthResponse>("/api/auth/login", { method: "POST", body }));
  } catch (error) {
    return toFormState(error);
  }

  redirect(safeRedirectTarget(field(data, "next")));
}

/** Creates a member account, then logs it in. */
export async function register(_: FormState, data: FormData): Promise<FormState> {
  const body: RegisterRequest = {
    username: field(data, "username"),
    email: field(data, "email"),
    password: field(data, "password"),
  };

  try {
    await storeSession(await apiFetch<AuthResponse>("/api/auth/register", { method: "POST", body }));
  } catch (error) {
    return toFormState(error);
  }

  redirect(safeRedirectTarget(field(data, "next")));
}

/** Deletes the session cookie and goes back to the blog. */
export async function logout(): Promise<void> {
  await clearSession();
  redirect("/blog");
}

// ------------------------------------------------------------ Administration

/** Promotes or demotes a user. */
export async function changeUserRole(id: string, role: RoleDto): Promise<FormState> {
  const body: ChangeRoleRequest = { role };

  try {
    await apiFetch<UserDto>(`/api/admin/users/${id}/role`, { method: "PATCH", body, auth: true });
  } catch (error) {
    return toFormState(error);
  }

  revalidatePath("/dashboard/users");
  return { success: true, message: `Role changed to ${role}.` };
}
