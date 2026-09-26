import type { ErrorBody } from "@/types/api/shared/ErrorBody";
import type { ErrorCode } from "@/types/api/shared/ErrorCode";

/**
 * Error thrown when the Rust API answers with a non-2xx status.
 * Mirrors the `ErrorBody` JSON returned by the backend.
 */
export class ApiError extends Error {
  readonly status: number;
  readonly code: ErrorCode;
  readonly fields: Record<string, string[]>;

  /** Builds the error from the HTTP status and the (possibly missing) JSON body. */
  constructor(status: number, body: Partial<ErrorBody> | null) {
    super(body?.message ?? `API request failed with status ${status}`);
    this.name = "ApiError";
    this.status = status;
    this.code = body?.error ?? "internal_error";
    this.fields = body?.fields ?? {};
  }
}

/** Tells whether `error` is an {@link ApiError} with the given status. */
export function isApiError(error: unknown, status?: number): error is ApiError {
  return error instanceof ApiError && (status === undefined || error.status === status);
}

/** Returns `null` instead of throwing when the API answers 404. */
export async function orNull<T>(request: Promise<T>): Promise<T | null> {
  try {
    return await request;
  } catch (error) {
    if (isApiError(error, 404)) {
      return null;
    }
    throw error;
  }
}
