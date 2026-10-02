import "server-only";

import { headers } from "next/headers";
import { unstable_rethrow } from "next/navigation";

/**
 * Returns the IP address of the visitor, forwarded to the Rust API on every
 * call (`X-Forwarded-For`) for the captcha check and the per-IP rate limits
 * (global, and stricter on login/register and the CV download).
 *
 * Outside of a request (e.g. at build time) there is no visitor: returns
 * `undefined` and the API falls back to the address of the Next.js server.
 * Errors Next.js uses for its own control flow are rethrown.
 *
 * `x-real-ip` and `x-forwarded-for` are client-controlled by nature: in
 * production, the reverse proxy in front of Next.js must overwrite them with
 * the real peer address, otherwise a bot could rotate fake IPs.
 */
export async function clientIp(): Promise<string | undefined> {
  let list: Awaited<ReturnType<typeof headers>>;
  try {
    list = await headers();
  } catch (error) {
    unstable_rethrow(error);
    return undefined;
  }
  const realIp = list.get("x-real-ip")?.trim();
  if (realIp) {
    return realIp;
  }
  return list.get("x-forwarded-for")?.split(",")[0]?.trim() || undefined;
}
