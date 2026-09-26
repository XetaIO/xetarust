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
