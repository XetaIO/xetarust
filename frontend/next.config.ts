import type { NextConfig } from "next";

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
