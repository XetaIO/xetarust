"use server";

import { revalidatePath } from "next/cache";
import { redirect } from "next/navigation";

import { apiFetch, apiUpload } from "@/lib/api/client";
import { field, type FormState, optionalField, toFormState } from "@/lib/forms";
import type { ArticleDto } from "@/types/api/publishing/ArticleDto";
import type { CategoryDto } from "@/types/api/publishing/CategoryDto";
import type { UpsertArticleRequest } from "@/types/api/publishing/UpsertArticleRequest";
import type { UpsertCategoryRequest } from "@/types/api/publishing/UpsertCategoryRequest";

// ------------------------------------------------------------------ Articles

/**
 * Creates (`id === null`) or updates an article, then applies the cover image
 * change (upload of the `cover` file or `remove_cover` checkbox) and returns
 * to the list.
 */
export async function saveArticle(id: string | null, _: FormState, data: FormData): Promise<FormState> {
  const body: UpsertArticleRequest = {
    category_id: field(data, "category_id"),
    title: field(data, "title"),
    slug: optionalField(data, "slug"),
    excerpt: optionalField(data, "excerpt"),
    content: field(data, "content"),
    publish: data.get("publish") === "on",
    comments_enabled: data.get("comments_enabled") === "on",
  };

  let saved: ArticleDto;
  try {
    saved = await apiFetch<ArticleDto>(id ? `/api/admin/articles/${id}` : "/api/admin/articles", {
      method: id ? "PUT" : "POST",
      body,
      auth: true,
    });
  } catch (error) {
    return toFormState(error);
  }

  const coverError = await saveCover(saved.id, data);
  revalidatePath("/blog", "layout");
  revalidatePath("/dashboard", "layout");
  if (coverError) {
    if (id) {
      return coverError;
    }
    // The article now exists: keep editing it instead of creating a duplicate.
    const message = coverError.fields?.cover?.join(", ") ?? coverError.message ?? "";
    redirect(`/dashboard/articles/${saved.id}/edit?cover_error=${encodeURIComponent(message)}`);
  }
  redirect("/dashboard/articles");
}

/**
 * Uploads the chosen cover image of an article, or removes the current one
 * when requested. Returns the form state of a rejected change, `null` otherwise.
 */
async function saveCover(articleId: string, data: FormData): Promise<FormState> {
  const cover = data.get("cover");
  const path = `/api/admin/articles/${articleId}/cover`;
  try {
    if (cover instanceof File && cover.size > 0) {
      await apiUpload<ArticleDto>(path, cover);
    } else if (data.get("remove_cover") === "on") {
      await apiFetch<ArticleDto>(path, { method: "DELETE", auth: true });
    }
  } catch (error) {
    return toFormState(error);
  }
  return null;
}

/** Deletes an article (its comments are removed by the API). */
export async function deleteArticle(id: string): Promise<FormState> {
  try {
    await apiFetch<void>(`/api/admin/articles/${id}`, { method: "DELETE", auth: true });
  } catch (error) {
    return toFormState(error);
  }

  revalidatePath("/blog", "layout");
  revalidatePath("/dashboard", "layout");
  return { success: true, message: "Article deleted." };
}

// ---------------------------------------------------------------- Categories

/** Creates (`id === null`) or updates a category. */
export async function saveCategory(id: string | null, _: FormState, data: FormData): Promise<FormState> {
  const body: UpsertCategoryRequest = {
    name: field(data, "name"),
    slug: optionalField(data, "slug"),
    description: optionalField(data, "description"),
  };

  try {
    await apiFetch<CategoryDto>(id ? `/api/admin/categories/${id}` : "/api/admin/categories", {
      method: id ? "PUT" : "POST",
      body,
      auth: true,
    });
  } catch (error) {
    return toFormState(error);
  }

  revalidatePath("/blog", "layout");
  revalidatePath("/dashboard/categories");
  return { success: true, message: id ? "Category updated." : "Category created." };
}

/** Deletes an empty category. */
export async function deleteCategory(id: string): Promise<FormState> {
  try {
    await apiFetch<void>(`/api/admin/categories/${id}`, { method: "DELETE", auth: true });
  } catch (error) {
    return toFormState(error);
  }

  revalidatePath("/blog", "layout");
  revalidatePath("/dashboard/categories");
  return { success: true, message: "Category deleted." };
}
