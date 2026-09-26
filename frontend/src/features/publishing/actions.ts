"use server";

import { revalidatePath } from "next/cache";
import { redirect } from "next/navigation";

import { apiFetch } from "@/lib/api/client";
import { field, type FormState, optionalField, toFormState } from "@/lib/forms";
import type { ArticleDto } from "@/types/api/publishing/ArticleDto";
import type { CategoryDto } from "@/types/api/publishing/CategoryDto";
import type { UpsertArticleRequest } from "@/types/api/publishing/UpsertArticleRequest";
import type { UpsertCategoryRequest } from "@/types/api/publishing/UpsertCategoryRequest";

// ------------------------------------------------------------------ Articles

/** Creates (`id === null`) or updates an article, then returns to the list. */
export async function saveArticle(id: string | null, _: FormState, data: FormData): Promise<FormState> {
  const body: UpsertArticleRequest = {
    category_id: field(data, "category_id"),
    title: field(data, "title"),
    slug: optionalField(data, "slug"),
    excerpt: optionalField(data, "excerpt"),
    content: field(data, "content"),
    publish: data.get("publish") === "on",
  };

  try {
    await apiFetch<ArticleDto>(id ? `/api/admin/articles/${id}` : "/api/admin/articles", {
      method: id ? "PUT" : "POST",
      body,
      auth: true,
    });
  } catch (error) {
    return toFormState(error);
  }

  revalidatePath("/blog", "layout");
  revalidatePath("/dashboard", "layout");
  redirect("/dashboard/articles");
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
