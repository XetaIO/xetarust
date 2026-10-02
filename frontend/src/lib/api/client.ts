import "server-only";

import { cookies } from "next/headers";

import { SESSION_COOKIE } from "@/lib/api/session-cookie";
import { clientIp } from "@/lib/client-ip";

import { ApiError } from "./errors";

/** Base URL of the Rust API. Only ever used server-side. */
const API_URL = process.env.API_URL ?? "http://127.0.0.1:8080";

type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

export interface ApiRequestOptions {
  method?: HttpMethod;
  /** Request body: a `Blob` is sent as-is (raw file), anything else as JSON. */
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
 * Returns the `X-Forwarded-For` header carrying the visitor IP, if known:
 * the API rate limits each visitor.
 */
async function forwardedForHeader(): Promise<Record<string, string>> {
  const ip = await clientIp();
  return ip ? { "X-Forwarded-For": ip } : {};
}

/**
 * Returns the body of an API call with its `Content-Type`: a `Blob` keeps its
 * own type (raw file upload), anything else is serialized as JSON.
 */
function requestBody(body: unknown): { body?: BodyInit; contentType?: string } {
  if (body === undefined) {
    return {};
  }
  if (body instanceof Blob) {
    return { body, contentType: body.type || "application/octet-stream" };
  }
  return { body: JSON.stringify(body), contentType: "application/json" };
}

/**
 * Builds the `fetch` init of an API call: method, body (JSON or raw file),
 * JWT (when `auth`) and visitor IP headers. Responses are never cached by
 * Next.js.
 */
async function requestInit(options: ApiRequestOptions, accept: string): Promise<RequestInit> {
  const { body, contentType } = requestBody(options.body);
  const headers: Record<string, string> = {
    Accept: accept,
    ...(contentType ? { "Content-Type": contentType } : {}),
    ...(options.auth ? await authorizationHeader() : {}),
    ...(await forwardedForHeader()),
  };

  return {
    method: options.method ?? "GET",
    headers,
    body,
    cache: "no-store",
  };
}

/**
 * Calls the Rust API from the Next.js server and returns the parsed JSON.
 * The JWT never reaches the browser: it is read from the httpOnly cookie here.
 *
 * @throws {ApiError} when the API answers with a non-2xx status.
 */
export async function apiFetch<T>(path: string, options: ApiRequestOptions = {}): Promise<T> {
  const response = await fetch(buildUrl(path, options.query), await requestInit(options, "application/json"));

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
 * Calls the Rust API from the Next.js server and returns the raw bytes of a
 * binary response (e.g. the CV as a PDF).
 *
 * @throws {ApiError} when the API answers with a non-2xx status (JSON body).
 */
export async function apiFetchBytes(path: string, options: ApiRequestOptions = {}): Promise<Uint8Array<ArrayBuffer>> {
  const response = await fetch(
    buildUrl(path, options.query),
    await requestInit(options, "application/pdf, application/json"),
  );

  if (!response.ok) {
    const data: unknown = await response.json().catch(() => null);
    throw new ApiError(response.status, data as ConstructorParameters<typeof ApiError>[1]);
  }
  return new Uint8Array(await response.arrayBuffer());
}

/**
 * Sends a raw file to the Rust API with `PUT` (JWT forwarded from the session
 * cookie) and returns the parsed JSON.
 *
 * @throws {ApiError} when the API answers with a non-2xx status.
 */
export async function apiUpload<T>(path: string, file: Blob): Promise<T> {
  return apiFetch<T>(path, { method: "PUT", body: file, auth: true });
}

/** Response headers of the API relayed as-is by {@link apiProxy}. */
const PROXIED_HEADERS = [
  "Content-Type",
  "Cache-Control",
  "Content-Length",
  "Content-Disposition",
  "X-Content-Type-Options",
];

/** Options of {@link apiProxy}: the proxied routes are public (no JWT). */
export type ApiProxyOptions = Pick<ApiRequestOptions, "method" | "body">;

/**
 * Relays a public, non-JSON API response (e.g. an image) to the
 * browser: same status, body and content headers. JSON errors are relayed
 * unchanged. Used by route handlers so the browser never talks to the Rust
 * API directly.
 */
export async function apiProxy(path: string, options: ApiProxyOptions = {}): Promise<Response> {
  const response = await fetch(buildUrl(path, undefined), await requestInit(options, "*/*"));
  const headers = new Headers();
  for (const name of PROXIED_HEADERS) {
    const value = response.headers.get(name);
    if (value) {
      headers.set(name, value);
    }
  }
  return new Response(response.body, { status: response.status, headers });
}
