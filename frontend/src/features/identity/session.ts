import "server-only";

import { cookies } from "next/headers";
import { notFound, redirect } from "next/navigation";
import { cache } from "react";

import { apiFetch } from "@/lib/api/client";
import { isApiError } from "@/lib/api/errors";
import { SESSION_COOKIE } from "@/lib/api/session-cookie";
import type { AuthResponse } from "@/types/api/identity/AuthResponse";
import type { UserDto } from "@/types/api/identity/UserDto";

/**
 * Stores the JWT in an httpOnly cookie so client-side JavaScript can never read it.
 * Must be called from a Server Action or a Route Handler.
 */
export async function storeSession(auth: AuthResponse): Promise<void> {
  (await cookies()).set(SESSION_COOKIE, auth.token, {
    httpOnly: true,
    secure: process.env.NODE_ENV === "production",
    sameSite: "lax",
    path: "/",
    expires: new Date(auth.expires_at),
  });
}

/** Removes the session cookie (logout). Server Actions / Route Handlers only. */
export async function clearSession(): Promise<void> {
  (await cookies()).delete(SESSION_COOKIE);
}

/**
 * Returns the logged-in user, or `null` when there is no valid session.
 * Memoized per request with `cache` so layouts and pages share one API call.
 */
export const getCurrentUser = cache(async (): Promise<UserDto | null> => {
  const token = (await cookies()).get(SESSION_COOKIE)?.value;
  if (!token) {
    return null;
  }

  try {
    return await apiFetch<UserDto>("/api/auth/me", { auth: true });
  } catch (error) {
    if (isApiError(error, 401)) {
      return null;
    }
    throw error;
  }
});

/** Returns the logged-in user or redirects to the login page. */
export async function requireUser(returnTo: string): Promise<UserDto> {
  const user = await getCurrentUser();
  if (!user) {
    redirect(`/login?next=${encodeURIComponent(returnTo)}`);
  }
  return user;
}

/** Returns the logged-in admin; members get a 404 so the dashboard stays hidden. */
export async function requireAdmin(): Promise<UserDto> {
  const user = await requireUser("/dashboard");
  if (user.role !== "admin") {
    notFound();
  }
  return user;
}
