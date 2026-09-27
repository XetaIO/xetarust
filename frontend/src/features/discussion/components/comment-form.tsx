"use client";

import { useActionState, useEffect, useRef } from "react";

import { FieldError, FormMessage } from "@/components/forms/field-error";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { postComment } from "@/features/discussion/actions";
import type { FormState } from "@/lib/forms";
import { Pencil, RotateCw } from "lucide-react";

/** Form posting a comment on the article `slug`. */
export function CommentForm({ slug }: { slug: string }) {
    const [state, action, pending] = useActionState<FormState, FormData>(postComment.bind(null, slug), null);
    const formRef = useRef<HTMLFormElement>(null);

    useEffect(() => {
        if (state?.success) {
            formRef.current?.reset();
        }
    }, [state]);

    return (
        <form ref={formRef} action={action} className="space-y-3">
            <Textarea
                name="content"
                rows={4}
                required
                minLength={2}
                maxLength={5000}
                placeholder="Share your thoughts…"
                aria-invalid={Boolean(state?.fields?.content)}
            />
            <FieldError messages={state?.fields?.content} />
            <FormMessage message={state?.message} success={state?.success} />
            <Button type="submit" disabled={pending}>
                {pending ? (
                    <>
                        <RotateCw className="animate-spin" /> Posting…
                    </>
                ) : (
                    <>
                        <Pencil /> Post comment
                    </>
                )}
            </Button>
        </form>
    );
}
