import "server-only";

import { headers } from "next/headers";

/**
 * Returns the IP address of the visitor, forwarded to the Rust API for the
 * captcha check and the per-IP rate limit of login/register and of the CV
 * download.
 *
 * `x-real-ip` and `x-forwarded-for` are client-controlled by nature: in
 * production, the reverse proxy in front of Next.js must overwrite them with
 * the real peer address, otherwise a bot could rotate fake IPs.
 */
export async function clientIp(): Promise<string | undefined> {
  const list = await headers();
  const realIp = list.get("x-real-ip")?.trim();
  if (realIp) {
    return realIp;
  }
  return list.get("x-forwarded-for")?.split(",")[0]?.trim() || undefined;
}
