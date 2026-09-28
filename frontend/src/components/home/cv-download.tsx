"use client";

import { Turnstile } from "@marsidev/react-turnstile";
import { Download, RotateCw } from "lucide-react";
import { useState } from "react";

import { FormMessage } from "@/components/forms/field-error";

/** Route handler relaying the CV download to the API (`src/app/cv/route.ts`). */
const CV_ENDPOINT = "/cv";

/** Name given to the downloaded file. */
const CV_FILENAME = "CV_Emeric_Fevre.pdf";

/** Message shown when the endpoint fails without a readable JSON body. */
const FALLBACK_ERROR = "the download failed, please try again";

/**
 * Public Turnstile site key, inlined at build time. Its presence is enforced
 * by `next.config.ts`, which refuses to start or build without it.
 */
const TURNSTILE_SITE_KEY = process.env.NEXT_PUBLIC_TURNSTILE_SITE_KEY ?? "";

type Status = "idle" | "verifying" | "downloading" | "error";

/** JSON error returned by the API (only the parts shown to the visitor). */
interface ApiErrorBody {
    message?: unknown;
    fields?: { captcha_token?: unknown };
}

/**
 * Reads an API error response: the `captcha_token` field error first, then
 * the global `message` (e.g. rate limit), falling back to a generic one.
 */
async function errorMessage(response: Response): Promise<string> {
    try {
        const body = (await response.json()) as ApiErrorBody;
        const captcha = body.fields?.captcha_token;
        if (Array.isArray(captcha) && typeof captcha[0] === "string") {
            return captcha[0];
        }
        return typeof body.message === "string" ? body.message : FALLBACK_ERROR;
    } catch {
        return FALLBACK_ERROR;
    }
}

/** Makes the browser save `blob` under `filename` through a temporary link. */
function saveBlob(blob: Blob, filename: string): void {
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = filename;
    document.body.append(link);
    link.click();
    link.remove();
    URL.revokeObjectURL(url);
}

/**
 * "Download my CV" button: the click reveals a Turnstile challenge (Cloudflare
 * is only loaded at that moment), and its token is exchanged for the PDF.
 */
export function CvDownload() {
    const [status, setStatus] = useState<Status>("idle");
    const [message, setMessage] = useState<string>();

    /** Exchanges the Turnstile `token` for the CV and saves it. */
    async function download(token: string) {
        setStatus("downloading");
        try {
            const response = await fetch(CV_ENDPOINT, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ captcha_token: token }),
            });
            if (!response.ok) {
                fail(await errorMessage(response));
                return;
            }
            saveBlob(await response.blob(), CV_FILENAME);
            setMessage(undefined);
            setStatus("idle");
        } catch {
            fail(FALLBACK_ERROR);
        }
    }

    /**
     * Shows `text` and unmounts the widget: a token is single-use, and the next
     * click mounts a fresh widget (resetting it in place would retry forever).
     */
    function fail(text: string) {
        setMessage(text);
        setStatus("error");
    }

    const busy = status === "verifying" || status === "downloading";

    return (
        <div className="flex flex-col items-stretch gap-3 sm:items-center">
            <button
                type="button"
                onClick={() => {
                    setMessage(undefined);
                    setStatus("verifying");
                }}
                disabled={busy}
                className="glass inline-flex items-center justify-center gap-2 rounded-full px-6 py-3 font-medium transition-colors hover:bg-white/10 disabled:cursor-wait disabled:opacity-70 hover:cursor-pointer"
            >
                {busy ? <RotateCw className="size-4 animate-spin" /> : <Download className="size-4" />} Download my CV
            </button>
            {busy && (
                <Turnstile
                    siteKey={TURNSTILE_SITE_KEY}
                    options={{ theme: "dark" }}
                    onSuccess={download}
                    onError={() => fail("the captcha could not be loaded, please try again")}
                />
            )}
            <FormMessage message={message} />
        </div>
    );
}
