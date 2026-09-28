import { Trash2 } from "lucide-react";
import Link from "next/link";

import { ConfirmAction } from "@/components/forms/confirm-action";
import { deleteComment } from "@/features/discussion/actions";
import { formatDate } from "@/lib/format";
import type { CommentDto } from "@/types/api/discussion/CommentDto";

import { CommentForm } from "./comment-form";

/** What Discussion needs to know about the logged-in reader (provided by the page). */
export interface CommentViewer {
    id: string;
    isAdmin: boolean;
}

interface CommentSectionProps {
    slug: string;
    comments: CommentDto[];
    /** Logged-in reader, `null` for visitors. */
    viewer: CommentViewer | null;
    /** Whether visitors can create an account (provided by the page). */
    canRegister: boolean;
}

/** Comments of an article; members can post, authors and admins can delete. */
export function CommentSection({ slug, comments, viewer, canRegister }: CommentSectionProps) {
    return (
        <section aria-labelledby="comments" className="mt-12 border-t border-white/10 pt-8 sm:mt-16 sm:pt-10">
            <h2 id="comments" className="text-2xl font-semibold">
                Comments <span className="text-muted-foreground">({comments.length})</span>
            </h2>

            <ul className="mt-6 space-y-4">
                {comments.map((comment) => (
                    <li key={comment.id} className="rounded-2xl border border-white/5 bg-card/50 p-4 sm:p-5">
                        <div className="flex items-start justify-between gap-4">
                            <p className="min-w-0 text-sm wrap-break-word">
                                <span className="font-medium">{comment.author.username}</span>
                                <span className="text-muted-foreground"> · {formatDate(comment.created_at)}</span>
                            </p>
                            {viewer && (viewer.isAdmin || viewer.id === comment.author.id) && (
                                <ConfirmAction
                                    action={deleteComment.bind(null, comment.id, slug)}
                                    title="Delete this comment?"
                                    description="This action cannot be undone."
                                >
                                    <Trash2 />
                                </ConfirmAction>
                            )}
                        </div>
                        <p className="mt-2 wrap-break-word whitespace-pre-line text-muted-foreground">
                            {comment.content}
                        </p>
                    </li>
                ))}
                {comments.length === 0 && <li className="text-muted-foreground">No comment yet — be the first!</li>}
            </ul>

            <div className="mt-8">
                {viewer ? (
                    <CommentForm slug={slug} />
                ) : (
                    <p className="rounded-2xl border border-dashed border-white/10 p-4 text-center text-muted-foreground sm:p-6">
                        <Link href={`/login?next=/blog/${slug}`} className="text-brand-orange hover:underline">
                            Log in
                        </Link>{" "}
                        {canRegister && (
                            <>
                                or{" "}
                                <Link
                                    href={`/register?next=/blog/${slug}`}
                                    className="text-brand-orange hover:underline"
                                >
                                    create an account
                                </Link>{" "}
                            </>
                        )}
                        to join the discussion.
                    </p>
                )}
            </div>
        </section>
    );
}
