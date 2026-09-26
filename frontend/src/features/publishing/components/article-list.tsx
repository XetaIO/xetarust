import type { ArticleSummaryDto } from "@/types/api/publishing/ArticleSummaryDto";
import type { Paginated } from "@/types/api/shared/Paginated";

import { ArticleCard } from "./article-card";
import { Pagination } from "@/components/site/pagination";

interface ArticleListProps {
    articles: Paginated<ArticleSummaryDto>;
    /** Base path the `?page=` parameter is appended to. */
    basePath: string;
}

/** Grid of article cards with its pagination. */
export function ArticleList({ articles, basePath }: ArticleListProps) {
    if (articles.items.length === 0) {
        return (
            <p className="rounded-2xl border border-dashed border-white/10 p-12 text-center text-muted-foreground">
                No article published here yet.
            </p>
        );
    }

    return (
        <>
            <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
                {articles.items.map((article) => (
                    <ArticleCard key={article.id} article={article} />
                ))}
            </div>
            <Pagination
                page={articles.page}
                totalPages={articles.total_pages}
                hrefFor={(page) => `${basePath}?page=${page}`}
            />
        </>
    );
}
