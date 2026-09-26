/** Returns the public URL of a cover image (served by the `/media/covers` proxy). */
export function coverUrl(name: string): string {
  return `/media/covers/${encodeURIComponent(name)}`;
}
