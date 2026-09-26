import "server-only";

import { cookies } from "next/headers";

import { SESSION_COOKIE } from "@/lib/api/session-cookie";

import { ApiError } from "./errors";

/** Base URL of the Rust API. Only ever used server-side. */
const API_URL = process.env.API_URL ?? "http://127.0.0.1:8080";

type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

export interface ApiRequestOptions {
  method?: HttpMethod;
  /** JSON-serializable request body. */
  body?: unknown;
  /** Query string parameters; `null`/`undefined` values are skipped. */
  query?: Record<string, string | number | null | undefined>;
  /** When true, forwards the JWT stored in the httpOnly session cookie. */
  auth?: boolean;
}

/** Builds the absolute API URL for `path` and its query parameters. */
function buildUrl(path: string, query: ApiRequestOptions["query"]): string {
  const url = new URL(path, API_URL);
  for (const [key, value] of Object.entries(query ?? {})) {
    if (value !== null && value !== undefined && value !== "") {
      url.searchParams.set(key, String(value));
    }
  }
  return url.toString();
}

/** Returns the `Authorization` header built from the session cookie, if any. */
async function authorizationHeader(): Promise<Record<string, string>> {
  const token = (await cookies()).get(SESSION_COOKIE)?.value;
  return token ? { Authorization: `Bearer ${token}` } : {};
}

/**
 * Calls the Rust API from the Next.js server and returns the parsed JSON.
 * The JWT never reaches the browser: it is read from the httpOnly cookie here.
 *
 * @throws {ApiError} when the API answers with a non-2xx status.
 */
export async function apiFetch<T>(path: string, options: ApiRequestOptions = {}): Promise<T> {
  const headers: Record<string, string> = {
    Accept: "application/json",
    ...(options.auth ? await authorizationHeader() : {}),
  };
  if (options.body !== undefined) {
    headers["Content-Type"] = "application/json";
  }

  const response = await fetch(buildUrl(path, options.query), {
    method: options.method ?? "GET",
    headers,
    body: options.body === undefined ? undefined : JSON.stringify(options.body),
    cache: "no-store",
  });

  if (response.status === 204) {
    return undefined as T;
  }

  const data: unknown = await response.json().catch(() => null);
  if (!response.ok) {
    throw new ApiError(response.status, data as ConstructorParameters<typeof ApiError>[1]);
  }
  return data as T;
}

/**
 * Sends a raw file to the Rust API with `PUT` (JWT forwarded from the session
 * cookie) and returns the parsed JSON.
 *
 * @throws {ApiError} when the API answers with a non-2xx status.
 */
export async function apiUpload<T>(path: string, file: Blob): Promise<T> {
  const response = await fetch(buildUrl(path, undefined), {
    method: "PUT",
    headers: {
      Accept: "application/json",
      "Content-Type": file.type || "application/octet-stream",
      ...(await authorizationHeader()),
    },
    body: file,
    cache: "no-store",
  });

  const data: unknown = await response.json().catch(() => null);
  if (!response.ok) {
    throw new ApiError(response.status, data as ConstructorParameters<typeof ApiError>[1]);
  }
  return data as T;
}

/** Response headers of the API relayed as-is by {@link apiProxy}. */
const PROXIED_HEADERS = ["Content-Type", "Cache-Control", "Content-Length"];

/**
 * Relays a public, non-JSON API response (e.g. an image) to the browser:
 * same status, body and caching headers. Used by route handlers so the
 * browser never talks to the Rust API directly.
 */
export async function apiProxy(path: string): Promise<Response> {
  const response = await fetch(buildUrl(path, undefined), { cache: "no-store" });
  const headers = new Headers();
  for (const name of PROXIED_HEADERS) {
    const value = response.headers.get(name);
    if (value) {
      headers.set(name, value);
    }
  }
  return new Response(response.body, { status: response.status, headers });
}
