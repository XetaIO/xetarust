import type { MetadataRoute } from "next";

import { coverUrl } from "@/features/publishing/cover";
import { getAllArticles, getCategories } from "@/features/publishing/queries";
import { absoluteUrl } from "@/lib/site";
import type { ArticleSummaryDto } from "@/types/api/publishing/ArticleSummaryDto";

// Built on every request: always up to date, and `next build` never needs the API.
export const dynamic = "force-dynamic";

/** Returns the most recent `updated_at` of `articles`, or `undefined` when empty. */
function lastUpdate(articles: ArticleSummaryDto[]): string | undefined {
  return articles.reduce<string | undefined>(
    (latest, article) => (latest && latest > article.updated_at ? latest : article.updated_at),
    undefined,
  );
}

/**
 * `/sitemap.xml`: the home page, the blog, every non-empty category and every
 * published article (with its cover image). Paginated listings, auth pages and
 * the dashboard are left out. An API failure surfaces as a 500 so crawlers
 * retry, instead of receiving a sitemap without the articles.
 */
export default async function sitemap(): Promise<MetadataRoute.Sitemap> {
  const [articles, categories] = await Promise.all([getAllArticles(), getCategories()]);

  const categoryEntries = categories.flatMap((category) => {
    const categoryArticles = articles.filter((article) => article.category.slug === category.slug);
    if (categoryArticles.length === 0) {
      return [];
    }
    return [
      {
        url: absoluteUrl(`/blog/category/${category.slug}`),
        lastModified: lastUpdate(categoryArticles),
        changeFrequency: "weekly" as const,
        priority: 0.6,
      },
    ];
  });

  const articleEntries = articles.map((article) => ({
    url: absoluteUrl(`/blog/${article.slug}`),
    lastModified: article.updated_at,
    changeFrequency: "monthly" as const,
    priority: 0.8,
    images: article.cover_image ? [absoluteUrl(coverUrl(article.cover_image))] : undefined,
  }));

  return [
    { url: absoluteUrl("/"), changeFrequency: "monthly", priority: 1 },
    { url: absoluteUrl("/blog"), lastModified: lastUpdate(articles), changeFrequency: "weekly", priority: 0.9 },
    ...categoryEntries,
    ...articleEntries,
  ];
}
