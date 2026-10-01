import "server-only";

import { apiFetch } from "@/lib/api/client";
import { orNull } from "@/lib/api/errors";
import type { ArticleDto } from "@/types/api/publishing/ArticleDto";
import type { ArticleSummaryDto } from "@/types/api/publishing/ArticleSummaryDto";
import type { ArticlesQuery } from "@/types/api/publishing/ArticlesQuery";
import type { CategoryDto } from "@/types/api/publishing/CategoryDto";
import type { PageQuery } from "@/types/api/shared/PageQuery";
import type { Paginated } from "@/types/api/shared/Paginated";

// ---------------------------------------------------------------- Public blog

/** Lists published articles (optionally filtered by category slug). */
export function getArticles(query: ArticlesQuery = {}): Promise<Paginated<ArticleSummaryDto>> {
  return apiFetch("/api/articles", { query: { ...query } });
}

/** Largest page size accepted by the API (`MAX_PER_PAGE` in `backend/kernel/src/pagination.rs`). */
const MAX_PER_PAGE = 50;

/**
 * Lists every published article by walking all the pages of the public listing:
 * the first page gives the page count, the others are fetched in parallel.
 */
export async function getAllArticles(): Promise<ArticleSummaryDto[]> {
  const first = await getArticles({ page: 1, per_page: MAX_PER_PAGE });
  const others = await Promise.all(
    Array.from({ length: Math.max(first.total_pages - 1, 0) }, (_, index) =>
      getArticles({ page: index + 2, per_page: MAX_PER_PAGE }),
    ),
  );
  return [first, ...others].flatMap((page) => page.items);
}

/** Loads a published article, or `null` when it does not exist. */
export function getArticle(slug: string): Promise<ArticleDto | null> {
  return orNull(apiFetch(`/api/articles/${encodeURIComponent(slug)}`));
}

/** Lists every category. */
export function getCategories(): Promise<CategoryDto[]> {
  return apiFetch("/api/categories");
}

// ------------------------------------------------------------- Administration

/** Lists every article, drafts included (admin only). */
export function getAdminArticles(query: PageQuery = {}): Promise<Paginated<ArticleSummaryDto>> {
  return apiFetch("/api/admin/articles", { query: { ...query }, auth: true });
}

/** Loads any article for edition (admin only), or `null` when missing. */
export function getAdminArticle(id: string): Promise<ArticleDto | null> {
  return orNull(apiFetch(`/api/admin/articles/${encodeURIComponent(id)}`, { auth: true }));
}
