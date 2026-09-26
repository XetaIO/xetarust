import type { Metadata } from "next";
import Link from "next/link";
import { notFound } from "next/navigation";

import { PageHeader } from "@/components/dashboard/page-header";
import { buttonVariants } from "@/components/ui/button";
import { ArticleForm } from "@/features/publishing/components/article-form";
import { getAdminArticle, getCategories } from "@/features/publishing/queries";

export const metadata: Metadata = { title: "Edit article" };

/** Page to edit an existing article (`?cover_error` shows a cover rejected at creation). */
export default async function EditArticlePage({ params, searchParams }: PageProps<"/dashboard/articles/[id]/edit">) {
    const { id } = await params;
    const { cover_error: coverError } = await searchParams;
    const [article, categories] = await Promise.all([getAdminArticle(id), getCategories()]);
    if (!article) {
        notFound();
    }

    return (
        <>
            <PageHeader
                title="Edit article"
                description={article.title}
                actions={
                    article.is_published && (
                        <Link href={`/blog/${article.slug}`} className={buttonVariants({ variant: "outline" })}>
                            View on the blog
                        </Link>
                    )
                }
            />
            <ArticleForm
                categories={categories}
                article={article}
                coverError={typeof coverError === "string" ? coverError : undefined}
            />
        </>
    );
}
