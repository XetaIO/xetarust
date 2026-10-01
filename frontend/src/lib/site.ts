/**
 * Public URL of the site, shared by the metadata, the canonical URLs,
 * `robots.txt` and `sitemap.xml`.
 */
export const SITE_URL = process.env.DOMAIN_URL ?? "https://xetaravel.com";

/** Returns the absolute public URL of `path` (e.g. `/blog` → `https://xetaravel.com/blog`). */
export function absoluteUrl(path: string): string {
  return new URL(path, SITE_URL).toString();
}
