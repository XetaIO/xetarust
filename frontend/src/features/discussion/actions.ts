"use server";

import { revalidatePath } from "next/cache";

import { apiFetch } from "@/lib/api/client";
import { field, type FormState, toFormState } from "@/lib/forms";
import type { CommentDto } from "@/types/api/discussion/CommentDto";
import type { CreateCommentRequest } from "@/types/api/discussion/CreateCommentRequest";
import type { DeletedCommentsDto } from "@/types/api/discussion/DeletedCommentsDto";

/** Posts a comment on the article `slug` as the logged-in user. */
export async function postComment(slug: string, _: FormState, data: FormData): Promise<FormState> {
  const body: CreateCommentRequest = { content: field(data, "content") };

  try {
    await apiFetch<CommentDto>(`/api/articles/${encodeURIComponent(slug)}/comments`, {
      method: "POST",
      body,
      auth: true,
    });
  } catch (error) {
    return toFormState(error);
  }

  revalidatePath(`/blog/${slug}`);
  return { success: true, message: "Comment posted." };
}

/** Deletes a comment (author or admin), then refreshes the article page. */
export async function deleteComment(id: string, slug: string): Promise<FormState> {
  try {
    await apiFetch<void>(`/api/comments/${encodeURIComponent(id)}`, { method: "DELETE", auth: true });
  } catch (error) {
    return toFormState(error);
  }

  revalidatePath(`/blog/${slug}`);
  return { success: true, message: "Comment deleted." };
}

/** Permanently deletes every comment written by `authorId` (admin only). */
export async function deleteAuthorComments(authorId: string): Promise<FormState> {
  let result: DeletedCommentsDto;
  try {
    result = await apiFetch<DeletedCommentsDto>(`/api/admin/users/${encodeURIComponent(authorId)}/comments`, {
      method: "DELETE",
      auth: true,
    });
  } catch (error) {
    return toFormState(error);
  }

  revalidatePath("/blog", "layout");
  return { success: true, message: `${result.deleted} comment(s) deleted.` };
}
