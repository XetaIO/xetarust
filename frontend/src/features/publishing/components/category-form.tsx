"use client";

import { useActionState, useEffect, useRef } from "react";
import { toast } from "sonner";

import { FieldError, FormMessage } from "@/components/forms/field-error";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { saveCategory } from "@/features/publishing/actions";
import type { FormState } from "@/lib/forms";
import type { CategoryDto } from "@/types/api/publishing/CategoryDto";

/** Inline form creating a category, or updating `category` when given. */
export function CategoryForm({ category }: { category?: CategoryDto }) {
    const [state, action, pending] = useActionState<FormState, FormData>(
        saveCategory.bind(null, category?.id ?? null),
        null,
    );
    const formRef = useRef<HTMLFormElement>(null);
    const errors = state?.fields ?? {};

    useEffect(() => {
        if (state?.success) {
            toast.success(state.message);
            if (!category) {
                formRef.current?.reset();
            }
        }
    }, [state, category]);

    return (
        <form ref={formRef} action={action} className="space-y-2">
            <div className="grid gap-2 sm:grid-cols-[1fr_1fr_2fr_auto]">
                <div>
                    <Input name="name" placeholder="Name" defaultValue={category?.name} required aria-label="Name" />
                    <FieldError messages={errors.name} />
                </div>
                <div>
                    <Input name="slug" placeholder="slug (optional)" defaultValue={category?.slug} aria-label="Slug" />
                    <FieldError messages={errors.slug} />
                </div>
                <div>
                    <Input
                        name="description"
                        placeholder="Description (optional)"
                        defaultValue={category?.description ?? ""}
                        aria-label="Description"
                    />
                    <FieldError messages={errors.description} />
                </div>
                <Button type="submit" variant={category ? "outline" : "default"} disabled={pending}>
                    {category ? "Save" : "Add"}
                </Button>
            </div>
            {!state?.success && <FormMessage message={state?.message} />}
        </form>
    );
}
