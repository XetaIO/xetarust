import "server-only";

import { apiFetch } from "@/lib/api/client";
import type { UserDto } from "@/types/api/identity/UserDto";
import type { PageQuery } from "@/types/api/shared/PageQuery";
import type { Paginated } from "@/types/api/shared/Paginated";

/** Lists registered users (admin only). */
export function getUsers(query: PageQuery = {}): Promise<Paginated<UserDto>> {
  return apiFetch("/api/admin/users", { query: { ...query }, auth: true });
}
