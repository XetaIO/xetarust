import type { Metadata } from "next";

import { ArticleList } from "@/features/publishing/components/article-list";
import { CategoryNav } from "@/features/publishing/components/category-nav";
import { getArticles, getCategories } from "@/features/publishing/queries";
import { parsePage } from "@/lib/format";

export const metadata: Metadata = { title: "Blog" };

/** Blog index: latest published articles. */
export default async function BlogPage({ searchParams }: PageProps<"/blog">) {
    const page = parsePage((await searchParams).page);
    const [articles, categories] = await Promise.all([getArticles({ page }), getCategories()]);

    return (
        <>
            <div className="mb-10">
                <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl">
                    The <span className="text-gradient">blog</span>
                </h1>
                <p className="mt-3 text-lg text-muted-foreground">
                    Notes on Rust, Laravel, architecture and everything I learn along the way.
                </p>
            </div>
            <div className="mb-10">
                <CategoryNav categories={categories} />
            </div>
            <ArticleList articles={articles} basePath="/blog" />
        </>
    );
}
