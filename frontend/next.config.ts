import type { NextConfig } from "next";

// The captcha of login/register is mandatory: fail fast (dev and build)
// rather than shipping forms that can never be submitted.
if (!process.env.NEXT_PUBLIC_TURNSTILE_SITE_KEY) {
  throw new Error(
    "NEXT_PUBLIC_TURNSTILE_SITE_KEY is not set: the Turnstile captcha is mandatory. " +
      "Use the Cloudflare test key 1x00000000000000000000AA in development and e2e " +
      "(see frontend/.env.example), and the real site key in production.",
  );
}

const nextConfig: NextConfig = {
  images: {
    // Only these local paths can be optimized by `next/image`.
    localPatterns: [
      { pathname: "/images/**", search: "" },
      { pathname: "/media/covers/**", search: "" },
    ],
  },
  experimental: {
    serverActions: {
      // Cover images are uploaded through a Server Action (API limit: 5 MB).
      bodySizeLimit: "6mb",
    },
  },
};

export default nextConfig;
