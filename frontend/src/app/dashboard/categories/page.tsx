import { Trash2 } from "lucide-react";
import type { Metadata } from "next";

import { PageHeader } from "@/components/dashboard/page-header";
import { ConfirmAction } from "@/components/forms/confirm-action";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { deleteCategory } from "@/features/publishing/actions";
import { CategoryForm } from "@/features/publishing/components/category-form";
import { getCategories } from "@/features/publishing/queries";

export const metadata: Metadata = { title: "Categories" };

/** Management of the blog categories. */
export default async function CategoriesPage() {
    const categories = await getCategories();

    return (
        <>
            <PageHeader title="Categories" description="Organize the articles of the blog." />

            <Card className="mb-8">
                <CardHeader>
                    <CardTitle>New category</CardTitle>
                </CardHeader>
                <CardContent>
                    <CategoryForm />
                </CardContent>
            </Card>

            <ul className="space-y-3">
                {categories.map((category) => (
                    <li key={category.id} className="flex items-start gap-2 rounded-xl border border-white/5 p-4">
                        <div className="flex-1">
                            <CategoryForm category={category} />
                        </div>
                        <ConfirmAction
                            action={deleteCategory.bind(null, category.id)}
                            title={`Delete "${category.name}"?`}
                            description="Only categories without articles can be deleted."
                        >
                            <Trash2 />
                        </ConfirmAction>
                    </li>
                ))}
                {categories.length === 0 && <li className="text-muted-foreground">No category yet.</li>}
            </ul>
        </>
    );
}
