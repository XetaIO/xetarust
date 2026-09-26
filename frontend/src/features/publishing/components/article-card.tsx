import { Clock } from "lucide-react";
import Link from "next/link";

import { ArticleCover } from "@/features/publishing/components/article-cover";
import { formatDate } from "@/lib/format";
import type { ArticleSummaryDto } from "@/types/api/publishing/ArticleSummaryDto";

/** Preview of an article in the blog listing: cover banner, then meta and excerpt. */
export function ArticleCard({ article }: { article: ArticleSummaryDto }) {
    return (
        <article className="group relative flex flex-col overflow-hidden rounded-2xl border border-white/5 bg-card/50 transition-all hover:-translate-y-1 hover:border-brand-orange/40 hover:shadow-xl hover:shadow-brand-orange/5">
            <ArticleCover
                name={article.cover_image}
                alt=""
                sizes="(min-width: 1024px) 33vw, (min-width: 768px) 50vw, 100vw"
                className="border-b border-white/5"
                imageClassName="transition-transform duration-500 group-hover:scale-[1.03]"
            />
            <div className="flex flex-1 flex-col p-6">
                <div className="flex items-center gap-3 text-xs text-muted-foreground">
                    <Link
                        href={`/blog/category/${article.category.slug}`}
                        className="relative z-10 rounded-full bg-brand-orange/15 px-2.5 py-1 font-medium text-brand-orange hover:bg-brand-orange/25"
                    >
                        {article.category.name}
                    </Link>
                    {article.published_at && (
                        <time dateTime={article.published_at}>{formatDate(article.published_at)}</time>
                    )}
                </div>
                <h2 className="mt-4 text-xl font-semibold tracking-tight">
                    <Link href={`/blog/${article.slug}`} className="after:absolute after:inset-0">
                        {article.title}
                    </Link>
                </h2>
                {article.excerpt && <p className="mt-2 line-clamp-3 text-muted-foreground">{article.excerpt}</p>}
                <div className="mt-auto flex items-center gap-3 pt-6 text-xs text-muted-foreground">
                    <span>by {article.author.username}</span>
                    <span className="inline-flex items-center gap-1">
                        <Clock className="size-3" /> {article.reading_time_minutes} min read
                    </span>
                </div>
            </div>
        </article>
    );
}
