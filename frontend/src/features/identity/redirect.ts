/** Keeps post-login redirects on this site only (prevents open redirects). */
export function safeRedirectTarget(target: unknown): string {
  return typeof target === "string" && target.startsWith("/") && !target.startsWith("//") ? target : "/blog";
}
