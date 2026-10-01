"use server";

import { apiFetchBytes } from "@/lib/api/client";
import { clientIp } from "@/lib/client-ip";
import { type FormState, toFormState } from "@/lib/forms";
import type { DownloadResumeRequest } from "@/types/api/resume/DownloadResumeRequest";

/** Result of {@link downloadResume}: the PDF, or the error to show. */
export type ResumeDownload = { pdf: Uint8Array<ArrayBuffer> } | FormState;

/**
 * Exchanges a solved Turnstile `captchaToken` for the CV. Being a Server
 * Action, it benefits from the native `Origin`/`Host` check of Next.js.
 */
export async function downloadResume(captchaToken: string): Promise<ResumeDownload> {
  const body: DownloadResumeRequest = { captcha_token: captchaToken };

  try {
    return { pdf: await apiFetchBytes("/api/cv", { method: "POST", body, clientIp: await clientIp() }) };
  } catch (error) {
    return toFormState(error);
  }
}
