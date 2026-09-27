import type { Metadata } from "next";
import { notFound } from "next/navigation";

import { ArticleList } from "@/features/publishing/components/article-list";
import { CategoryNav } from "@/features/publishing/components/category-nav";
import { getArticles, getCategories } from "@/features/publishing/queries";
import { parsePage } from "@/lib/format";

/** Uses the category name as page title. */
export async function generateMetadata({ params }: PageProps<"/blog/category/[slug]">): Promise<Metadata> {
    const { slug } = await params;
    const category = (await getCategories()).find((c) => c.slug === slug);
    return { title: category?.name ?? "Category" };
}

/** Published articles of one category. */
export default async function CategoryPage({ params, searchParams }: PageProps<"/blog/category/[slug]">) {
    const { slug } = await params;
    const page = parsePage((await searchParams).page);
    const categories = await getCategories();
    const category = categories.find((c) => c.slug === slug);
    if (!category) {
        notFound();
    }

    const articles = await getArticles({ page, category: slug });

    return (
        <>
            <div className="mb-10">
                <p className="text-xs tracking-[0.3em] text-brand-amber uppercase">Category</p>
                <h1 className="mt-2 text-4xl font-semibold tracking-tight">{category.name}</h1>
                {category.description && <p className="mt-3 text-lg text-muted-foreground">{category.description}</p>}
            </div>
            <div className="mb-10">
                <CategoryNav categories={categories} active={slug} />
            </div>
            <ArticleList articles={articles} basePath={`/blog/category/${slug}`} />
        </>
    );
}
