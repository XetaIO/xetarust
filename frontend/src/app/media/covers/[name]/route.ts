import { apiProxy } from "@/lib/api/client";

/** Serves an article cover image, proxied from the Rust API. */
export async function GET(_: Request, { params }: RouteContext<"/media/covers/[name]">): Promise<Response> {
  const { name } = await params;
  return apiProxy(`/api/covers/${encodeURIComponent(name)}`);
}
