import { isApiError } from "@/lib/api/errors";

/** State returned by every Server Action used with `useActionState`. */
export type FormState = {
  /** Global message (error or success). */
  message?: string;
  /** Per-field validation messages coming from the Rust API. */
  fields?: Record<string, string[]>;
  success?: boolean;
} | null;

/** Converts an error thrown by `apiFetch` into a form state; rethrows unexpected errors. */
export function toFormState(error: unknown): FormState {
  if (isApiError(error) && error.status < 500) {
    return { message: capitalize(error.message), fields: error.fields };
  }
  throw error;
}

/** Reads a trimmed string field from a form. */
export function field(data: FormData, name: string): string {
  const value = data.get(name);
  return typeof value === "string" ? value.trim() : "";
}

/** Reads an optional string field: blank values become `null`. */
export function optionalField(data: FormData, name: string): string | null {
  return field(data, name) || null;
}

/** Uppercases the first letter of a message coming from the API. */
export function capitalize(message: string): string {
  return message.charAt(0).toUpperCase() + message.slice(1);
}
