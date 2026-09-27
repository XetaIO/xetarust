import { FileText, FolderTree, PenLine, Users } from "lucide-react";
import Link from "next/link";

import { PageHeader } from "@/components/dashboard/page-header";
import { buttonVariants } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { getUsers } from "@/features/identity/queries";
import { getCurrentUser } from "@/features/identity/session";
import { getAdminArticles, getCategories } from "@/features/publishing/queries";
import { formatDate } from "@/lib/format";

/** Dashboard overview: counters and latest articles. */
export default async function DashboardPage() {
    const [articles, categories, users, me] = await Promise.all([
        getAdminArticles({ per_page: 5 }),
        getCategories(),
        getUsers({ per_page: 1 }),
        getCurrentUser(),
    ]);

    const stats = [
        { label: "Articles", value: articles.total, icon: FileText, href: "/dashboard/articles" },
        { label: "Categories", value: categories.length, icon: FolderTree, href: "/dashboard/categories" },
        { label: "Users", value: users.total, icon: Users, href: "/dashboard/users" },
    ];

    return (
        <>
            <PageHeader
                title={`Hello, ${me?.username ?? "admin"}`}
                description="Here is what is happening on your site."
                actions={
                    <Link href="/dashboard/articles/new" className={buttonVariants()}>
                        <PenLine /> Write an article
                    </Link>
                }
            />

            <div className="grid gap-4 sm:grid-cols-3">
                {stats.map((stat) => (
                    <Link key={stat.label} href={stat.href}>
                        <Card className="transition-colors hover:border-brand-orange/40">
                            <CardHeader className="flex flex-row items-center justify-between">
                                <CardTitle className="text-sm font-medium text-muted-foreground">
                                    {stat.label}
                                </CardTitle>
                                <stat.icon className="size-4 text-muted-foreground" />
                            </CardHeader>
                            <CardContent className="text-3xl font-semibold">{stat.value}</CardContent>
                        </Card>
                    </Link>
                ))}
            </div>

            <h2 className="mt-12 mb-4 text-lg font-semibold">Latest articles</h2>
            <ul className="divide-y divide-white/5 rounded-xl border border-white/5">
                {articles.items.map((article) => (
                    <li key={article.id} className="flex items-center justify-between gap-4 p-4">
                        <Link
                            href={`/dashboard/articles/${article.id}/edit`}
                            className="min-w-0 truncate font-medium hover:underline"
                        >
                            {article.title}
                        </Link>
                        <span className="shrink-0 text-sm text-muted-foreground">
                            {article.is_published ? formatDate(article.published_at ?? article.created_at) : "Draft"}
                        </span>
                    </li>
                ))}
                {articles.items.length === 0 && <li className="p-4 text-muted-foreground">No article yet.</li>}
            </ul>
        </>
    );
}
