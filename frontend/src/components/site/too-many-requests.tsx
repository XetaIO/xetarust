"use client";

import { Button } from "@/components/ui/button";

interface TooManyRequestsProps {
    /** Re-fetches and re-renders the failed segment. */
    retry: () => void;
}

/**
 * Rate limit message (API 429), shown by the error boundaries instead of a
 * generic error. A plain block: the boundary decides of the surrounding `<main>`.
 */
export function TooManyRequests({ retry }: TooManyRequestsProps) {
    return (
        <div className="flex flex-col items-center py-24 text-center">
            <p className="text-brand-orange text-8xl font-bold">429</p>
            <h1 className="mt-4 text-2xl font-semibold">Slow down a little.</h1>
            <p className="mt-2 text-muted-foreground">
                Too many requests in a short time. Wait a few seconds, then try again.
            </p>
            <Button className="mt-8" onClick={retry}>
                Try again
            </Button>
        </div>
    );
}
