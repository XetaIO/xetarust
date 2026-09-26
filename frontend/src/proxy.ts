import { type NextRequest, NextResponse } from "next/server";

import { SESSION_COOKIE } from "@/lib/api/session-cookie";

/**
 * Optimistic guard of the dashboard: visitors without a session cookie are
 * sent to the login page. The real check (valid token + admin role) happens
 * server-side in the dashboard layout through the Rust API.
 */
export function proxy(request: NextRequest) {
  if (request.cookies.has(SESSION_COOKIE)) {
    return NextResponse.next();
  }

  const login = new URL("/login", request.url);
  login.searchParams.set("next", request.nextUrl.pathname);
  return NextResponse.redirect(login);
}

export const config = {
  matcher: ["/dashboard/:path*"],
};
