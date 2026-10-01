import { Pencil, Plus, Trash2 } from "lucide-react";
import type { Metadata } from "next";
import Link from "next/link";

import { PageHeader } from "@/components/dashboard/page-header";
import { ConfirmAction } from "@/components/forms/confirm-action";
import { Pagination } from "@/components/site/pagination";
import { Badge } from "@/components/ui/badge";
import { buttonVariants } from "@/components/ui/button";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { requireAdmin } from "@/features/identity/session";
import { deleteArticle } from "@/features/publishing/actions";
import { getAdminArticles } from "@/features/publishing/queries";
import { formatDate, parsePage } from "@/lib/format";

export const metadata: Metadata = { title: "Articles" };

/** List of every article, drafts included. */
export default async function DashboardArticlesPage({ searchParams }: PageProps<"/dashboard/articles">) {
    await requireAdmin();
    const page = parsePage((await searchParams).page);
    const articles = await getAdminArticles({ page, per_page: 20 });

    return (
        <>
            <PageHeader
                title="Articles"
                description={`${articles.total} article(s)`}
                actions={
                    <Link href="/dashboard/articles/new" className={buttonVariants()}>
                        <Plus /> New article
                    </Link>
                }
            />
            <div className="rounded-xl border border-white/5">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>Title</TableHead>
                            <TableHead className="hidden md:table-cell">Category</TableHead>
                            <TableHead>Status</TableHead>
                            <TableHead className="hidden lg:table-cell">Updated</TableHead>
                            <TableHead className="text-right">Actions</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {articles.items.map((article) => (
                            <TableRow key={article.id}>
                                <TableCell className="w-full max-w-0 truncate font-medium">{article.title}</TableCell>
                                <TableCell className="hidden md:table-cell">{article.category.name}</TableCell>
                                <TableCell>
                                    {article.is_published ? (
                                        <Badge>Published</Badge>
                                    ) : (
                                        <Badge variant="secondary">Draft</Badge>
                                    )}
                                </TableCell>
                                <TableCell className="hidden text-muted-foreground lg:table-cell">
                                    {formatDate(article.updated_at)}
                                </TableCell>
                                <TableCell>
                                    <div className="flex justify-end gap-2">
                                        <Link
                                            href={`/dashboard/articles/${article.id}/edit`}
                                            className={buttonVariants({ variant: "outline", size: "sm" })}
                                        >
                                            <Pencil /> <span className="sr-only sm:not-sr-only">Edit</span>
                                        </Link>
                                        <ConfirmAction
                                            action={deleteArticle.bind(null, article.id)}
                                            title={`Delete "${article.title}"?`}
                                            description="The article and all its comments will be permanently deleted."
                                        >
                                            <Trash2 />
                                        </ConfirmAction>
                                    </div>
                                </TableCell>
                            </TableRow>
                        ))}
                        {articles.items.length === 0 && (
                            <TableRow>
                                <TableCell colSpan={5} className="py-10 text-center text-muted-foreground">
                                    No article yet.
                                </TableCell>
                            </TableRow>
                        )}
                    </TableBody>
                </Table>
            </div>
            <Pagination
                page={articles.page}
                totalPages={articles.total_pages}
                hrefFor={(target) => `/dashboard/articles?page=${target}`}
            />
        </>
    );
}
