"use client";

import Link from "next/link";
import { type ChangeEvent, useActionState, useEffect, useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

import { FieldError, FormMessage } from "@/components/forms/field-error";
import { Button, buttonVariants } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { saveArticle } from "@/features/publishing/actions";
import { coverUrl } from "@/features/publishing/cover";
import type { FormState } from "@/lib/forms";
import { cn } from "@/lib/utils";
import type { ArticleDto } from "@/types/api/publishing/ArticleDto";
import type { CategoryDto } from "@/types/api/publishing/CategoryDto";

interface ArticleFormProps {
    categories: CategoryDto[];
    /** Article being edited; `undefined` when writing a new one. */
    article?: ArticleDto;
    /** Cover error to display initially (cover rejected right after creation). */
    coverError?: string;
}

/** Editor of an article with a live Markdown preview. */
export function ArticleForm({ categories, article, coverError }: ArticleFormProps) {
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

            <CoverField
                current={article?.cover_image ?? null}
                errors={errors.cover ?? (state === null && coverError ? [coverError] : undefined)}
            />

            <div className="space-y-2">
                <Label htmlFor="excerpt">Excerpt</Label>
                <Textarea id="excerpt" name="excerpt" rows={2} defaultValue={article?.excerpt ?? ""} maxLength={500} />
                <FieldError messages={errors.excerpt} />
            </div>

            <div className="space-y-2">
                <div className="flex flex-wrap items-center justify-between gap-2">
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
                    className={cn("min-h-40 text-sm sm:min-h-80", tab === "preview" && "hidden")}
                    aria-invalid={!!errors.content}
                />
                {tab === "preview" && (
                    <div className="prose prose-sm prose-invert min-h-80 max-w-none overflow-x-auto rounded-lg border border-white/10 p-4 sm:prose-base sm:min-h-128 sm:p-6">
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

            <div className="flex flex-col-reverse gap-3 sm:flex-row">
                <Button type="submit" size="lg" disabled={pending} className="w-full sm:w-auto">
                    {pending ? "Saving…" : article ? "Save changes" : "Create article"}
                </Button>
                <Link
                    href="/dashboard/articles"
                    className={buttonVariants({ variant: "ghost", size: "lg", className: "w-full sm:w-auto" })}
                >
                    Cancel
                </Link>
            </div>
        </form>
    );
}

interface CoverFieldProps {
    /** File name of the current cover image, if any. */
    current: string | null;
    errors?: string[];
}

/** File input of the cover image with a 16:9 preview and a "remove" option. */
function CoverField({ current, errors }: CoverFieldProps) {
    const [selected, setSelected] = useState<string | null>(null);
    const [remove, setRemove] = useState(false);

    useEffect(() => {
        return () => {
            if (selected) {
                URL.revokeObjectURL(selected);
            }
        };
    }, [selected]);

    /** Previews the newly chosen file (or falls back to the current cover). */
    function handleChange(event: ChangeEvent<HTMLInputElement>) {
        const file = event.target.files?.[0];
        setSelected(file ? URL.createObjectURL(file) : null);
    }

    const preview = selected ?? (current && !remove ? coverUrl(current) : null);

    return (
        <div className="space-y-2">
            <Label htmlFor="cover">Cover image</Label>
            <div className="grid gap-4 md:grid-cols-[16rem_1fr] md:items-start">
                <div className="relative aspect-video overflow-hidden rounded-lg border border-white/10 bg-card">
                    {preview ? (
                        // eslint-disable-next-line @next/next/no-img-element -- blob: previews cannot go through next/image
                        <img src={preview} alt="Cover preview" className="size-full object-cover" />
                    ) : (
                        <span className="flex size-full items-center justify-center text-xs text-muted-foreground">
                            No cover
                        </span>
                    )}
                </div>
                <div className="space-y-3">
                    <Input
                        id="cover"
                        name="cover"
                        type="file"
                        accept="image/jpeg,image/png,image/webp"
                        onChange={handleChange}
                        aria-invalid={!!errors}
                    />
                    <p className="text-xs text-muted-foreground">JPEG, PNG or WebP, 5 MB max. Displayed in 16:9.</p>
                    {current && (
                        <label className="flex items-center gap-3 text-sm">
                            <input
                                type="checkbox"
                                name="remove_cover"
                                checked={remove}
                                onChange={(event) => setRemove(event.target.checked)}
                                disabled={selected !== null}
                                className="size-4 accent-brand-orange"
                            />
                            Remove cover
                        </label>
                    )}
                    <FieldError messages={errors} />
                </div>
            </div>
        </div>
    );
}
