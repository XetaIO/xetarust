import "server-only";

import { apiFetch } from "@/lib/api/client";
import type { CommentDto } from "@/types/api/discussion/CommentDto";

/** Lists the comments of a published article. */
export function getComments(slug: string): Promise<CommentDto[]> {
  return apiFetch(`/api/articles/${encodeURIComponent(slug)}/comments`);
}
