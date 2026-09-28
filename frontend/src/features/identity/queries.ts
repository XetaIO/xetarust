import "server-only";

import { cache } from "react";

import { apiFetch } from "@/lib/api/client";
import type { IdentitySettingsDto } from "@/types/api/identity/IdentitySettingsDto";
import type { UserDto } from "@/types/api/identity/UserDto";
import type { PageQuery } from "@/types/api/shared/PageQuery";
import type { Paginated } from "@/types/api/shared/Paginated";

/** Lists registered users (admin only). */
export function getUsers(query: PageQuery = {}): Promise<Paginated<UserDto>> {
  return apiFetch("/api/admin/users", { query: { ...query }, auth: true });
}

/**
 * Returns the public settings of Identity (are registrations open?).
 * Memoized per request with `cache`; if the API fails, registrations are
 * considered open so the header never breaks.
 */
export const getIdentitySettings = cache(async (): Promise<IdentitySettingsDto> => {
  try {
    return await apiFetch<IdentitySettingsDto>("/api/settings/identity");
  } catch {
    return { registration_enabled: true };
  }
});
