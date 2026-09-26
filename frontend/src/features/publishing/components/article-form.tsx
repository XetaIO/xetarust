"use client";

import Link from "next/link";
import { useActionState, useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

import { FieldError, FormMessage } from "@/components/forms/field-error";
import { Button, buttonVariants } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { saveArticle } from "@/features/publishing/actions";
import type { FormState } from "@/lib/forms";
import { cn } from "@/lib/utils";
import type { ArticleDto } from "@/types/api/publishing/ArticleDto";
import type { CategoryDto } from "@/types/api/publishing/CategoryDto";

interface ArticleFormProps {
    categories: CategoryDto[];
    /** Article being edited; `undefined` when writing a new one. */
    article?: ArticleDto;
}

/** Editor of an article with a live Markdown preview. */
export function ArticleForm({ categories, article }: ArticleFormProps) {
    const [state, action, pending] = useActionState<FormState, FormData>(
        saveArticle.bind(null, article?.id ?? null),
        null,
    );
    const [content, setContent] = useState(article?.content ?? "");
    const [tab, setTab] = useState<"write" | "preview">("write");
    const errors = state?.fields ?? {};

    if (categories.length === 0) {
        return (
            <p className="rounded-xl border border-dashed border-white/10 p-8 text-center text-muted-foreground">
                Create a{" "}
                <Link href="/dashboard/categories" className="text-brand-amber hover:underline">
                    category
                </Link>{" "}
                before writing your first article.
            </p>
        );
    }

    return (
        <form action={action} className="space-y-6">
            <FormMessage message={state?.message} />

            <div className="grid gap-6 md:grid-cols-2">
                <div className="space-y-2">
                    <Label htmlFor="title">Title</Label>
                    <Input
                        id="title"
                        name="title"
                        defaultValue={article?.title}
                        required
                        aria-invalid={!!errors.title}
                    />
                    <FieldError messages={errors.title} />
                </div>
                <div className="space-y-2">
                    <Label htmlFor="slug">Slug</Label>
                    <Input
                        id="slug"
                        name="slug"
                        defaultValue={article?.slug}
                        placeholder="Generated from the title when empty"
                        aria-invalid={!!errors.slug}
                    />
                    <FieldError messages={errors.slug} />
                </div>
            </div>

            <div className="space-y-2">
                <Label htmlFor="category_id">Category</Label>
                <select
                    id="category_id"
                    name="category_id"
                    defaultValue={article?.category.id ?? categories[0].id}
                    className="h-9 w-full rounded-lg border border-input bg-transparent px-3 text-sm dark:bg-input/30"
                >
                    {categories.map((category) => (
                        <option key={category.id} value={category.id} className="bg-popover">
                            {category.name}
                        </option>
                    ))}
                </select>
                <FieldError messages={errors.category_id} />
            </div>

            <div className="space-y-2">
                <Label htmlFor="excerpt">Excerpt</Label>
                <Textarea id="excerpt" name="excerpt" rows={2} defaultValue={article?.excerpt ?? ""} maxLength={500} />
                <FieldError messages={errors.excerpt} />
            </div>

            <div className="space-y-2">
                <div className="flex items-center justify-between">
                    <Label htmlFor="content">Content (Markdown)</Label>
                    <div className="flex rounded-lg bg-white/5 p-0.5 text-sm">
                        {(["write", "preview"] as const).map((value) => (
                            <button
                                key={value}
                                type="button"
                                onClick={() => setTab(value)}
                                className={cn(
                                    "rounded-md px-3 py-1 capitalize",
                                    tab === value ? "bg-background text-foreground" : "text-muted-foreground",
                                )}
                            >
                                {value}
                            </button>
                        ))}
                    </div>
                </div>
                <Textarea
                    id="content"
                    name="content"
                    rows={22}
                    value={content}
                    onChange={(event) => setContent(event.target.value)}
                    className={cn("font-mono text-sm", tab === "preview" && "hidden")}
                    aria-invalid={!!errors.content}
                />
                {tab === "preview" && (
                    <div className="prose prose-invert min-h-96 max-w-none rounded-lg border border-white/10 p-6">
                        <ReactMarkdown remarkPlugins={[remarkGfm]}>{content || "*Nothing to preview.*"}</ReactMarkdown>
                    </div>
                )}
                <FieldError messages={errors.content} />
            </div>

            <label className="flex items-center gap-3 text-sm">
                <input
                    type="checkbox"
                    name="publish"
                    defaultChecked={article?.is_published ?? false}
                    className="size-4 accent-brand-orange"
                />
                Publish this article
            </label>

            <div className="flex gap-3">
                <Button type="submit" size="lg" disabled={pending}>
                    {pending ? "Saving…" : article ? "Save changes" : "Create article"}
                </Button>
                <Link href="/dashboard/articles" className={buttonVariants({ variant: "ghost", size: "lg" })}>
                    Cancel
                </Link>
            </div>
        </form>
    );
}
