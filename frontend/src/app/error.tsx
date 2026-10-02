"use client";

import { TooManyRequests } from "@/components/site/too-many-requests";
import { Button } from "@/components/ui/button";
import { isTooManyRequests } from "@/lib/api/errors";

/**
 * Error boundary of every page (the root layout itself never calls the API):
 * a dedicated page for the API rate limit, a generic message otherwise.
 */
export default function RootError({ error, retry }: { error: Error & { digest?: string }; retry: () => void }) {
    return (
        <main className="flex flex-1 flex-col items-center justify-center px-6">
            {isTooManyRequests(error) ? (
                <TooManyRequests retry={retry} />
            ) : (
                <div className="py-24 text-center">
                    <h1 className="text-2xl font-semibold">Something went wrong</h1>
                    <p className="mt-2 text-muted-foreground">This page is temporarily unavailable. Please try again.</p>
                    <Button className="mt-8" onClick={retry}>
                        Try again
                    </Button>
                </div>
            )}
        </main>
    );
}
