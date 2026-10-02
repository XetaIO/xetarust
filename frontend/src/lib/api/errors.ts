import type { ErrorBody } from "@/types/api/shared/ErrorBody";
import type { ErrorCode } from "@/types/api/shared/ErrorCode";

/**
 * Digest of a rendering error caused by the API rate limit (429). Next.js keeps
 * a digest set on the thrown error, and the digest is all an error boundary
 * receives in production: it is how `error.tsx` recognizes a rate limit.
 */
export const TOO_MANY_REQUESTS_DIGEST = "XETARAVEL_TOO_MANY_REQUESTS";

/**
 * Error thrown when the Rust API answers with a non-2xx status.
 * Mirrors the `ErrorBody` JSON returned by the backend.
 */
export class ApiError extends Error {
  readonly status: number;
  readonly code: ErrorCode;
  readonly fields: Record<string, string[]>;
  /** Fixed digest for a 429 ({@link TOO_MANY_REQUESTS_DIGEST}), else left to Next.js. */
  readonly digest?: string;

  /** Builds the error from the HTTP status and the (possibly missing) JSON body. */
  constructor(status: number, body: Partial<ErrorBody> | null) {
    super(body?.message ?? `API request failed with status ${status}`);
    this.name = "ApiError";
    this.status = status;
    this.code = body?.error ?? "internal_error";
    this.fields = body?.fields ?? {};
    if (status === 429) {
      this.digest = TOO_MANY_REQUESTS_DIGEST;
    }
  }
}

/** Tells whether the error received by an error boundary comes from the API rate limit. */
export function isTooManyRequests(error: { digest?: string }): boolean {
  return error.digest === TOO_MANY_REQUESTS_DIGEST;
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
