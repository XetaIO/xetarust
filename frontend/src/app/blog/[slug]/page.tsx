import { ArrowLeft, Clock } from "lucide-react";
import type { Metadata } from "next";
import Link from "next/link";
import { notFound } from "next/navigation";

import { CommentSection } from "@/features/discussion/components/comment-section";
import { getComments } from "@/features/discussion/queries";
import { getCurrentUser } from "@/features/identity/session";
import { Markdown } from "@/features/publishing/components/markdown";
import { getArticle } from "@/features/publishing/queries";
import { formatDate } from "@/lib/format";

/** Uses the article title and excerpt as metadata. */
export async function generateMetadata({ params }: PageProps<"/blog/[slug]">): Promise<Metadata> {
    const article = await getArticle((await params).slug);
    return article ? { title: article.title, description: article.excerpt ?? undefined } : {};
}

/** A published article with its comments. */
export default async function ArticlePage({ params }: PageProps<"/blog/[slug]">) {
    const { slug } = await params;
    const article = await getArticle(slug);
    if (!article) {
        notFound();
    }
    const [comments, user] = await Promise.all([getComments(slug), getCurrentUser()]);

    return (
        <article className="mx-auto max-w-3xl">
            <Link
                href="/blog"
                className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
            >
                <ArrowLeft className="size-4" /> All articles
            </Link>

            <header className="mt-8 mb-10">
                <Link
                    href={`/blog/category/${article.category.slug}`}
                    className="rounded-full bg-brand-orange/15 px-3 py-1 text-xs font-medium text-brand-orange"
                >
                    {article.category.name}
                </Link>
                <h1 className="mt-5 text-4xl font-semibold tracking-tight text-balance sm:text-5xl">{article.title}</h1>
                {article.excerpt && <p className="mt-4 text-xl text-muted-foreground">{article.excerpt}</p>}
                <p className="mt-6 flex flex-wrap items-center gap-3 text-sm text-muted-foreground">
                    <span>
                        by <span className="text-foreground">{article.author.username}</span>
                    </span>
                    {article.published_at && (
                        <time dateTime={article.published_at}>{formatDate(article.published_at)}</time>
                    )}
                    <span className="inline-flex items-center gap-1">
                        <Clock className="size-3.5" /> {article.reading_time_minutes} min read
                    </span>
                </p>
            </header>

            <Markdown source={article.content} />

            <CommentSection
                slug={slug}
                comments={comments}
                viewer={user && { id: user.id, isAdmin: user.role === "admin" }}
            />
        </article>
    );
}
