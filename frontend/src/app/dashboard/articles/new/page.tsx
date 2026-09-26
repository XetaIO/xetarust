import type { Metadata } from "next";

import { PageHeader } from "@/components/dashboard/page-header";
import { ArticleForm } from "@/features/publishing/components/article-form";
import { getCategories } from "@/features/publishing/queries";

export const metadata: Metadata = { title: "New article" };

/** Page to write a new article. */
export default async function NewArticlePage() {
    const categories = await getCategories();

    return (
        <>
            <PageHeader title="New article" />
            <ArticleForm categories={categories} />
        </>
    );
}
