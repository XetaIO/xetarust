import { clientIp } from "@/features/identity/client-ip";
import { apiProxy } from "@/lib/api/client";

/** Reads the `captcha_token` of the JSON body; anything else is left to the API to reject. */
async function captchaToken(request: Request): Promise<unknown> {
  try {
    const body: unknown = await request.json();
    return body && typeof body === "object" && "captcha_token" in body ? body.captcha_token : undefined;
  } catch {
    // Invalid JSON: treated as a missing token.
    return undefined;
  }
}

/**
 * Relays the CV download to the Rust API, which checks the Turnstile token
 * and sends the PDF as an attachment (or a JSON error, relayed unchanged).
 * Only POST is exported: `GET /cv` answers 405.
 */
export async function POST(request: Request): Promise<Response> {
  const token = await captchaToken(request);
  return apiProxy("/api/cv", {
    method: "POST",
    body: token === undefined ? {} : { captcha_token: token },
    clientIp: await clientIp(),
  });
}
