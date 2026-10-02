"use client";

import { TooManyRequests } from "@/components/site/too-many-requests";
import { Button } from "@/components/ui/button";
import { isTooManyRequests } from "@/lib/api/errors";

/** Error boundary of the blog: rate limit page, or the API is unreachable. */
export default function BlogError({ error, retry }: { error: Error & { digest?: string }; retry: () => void }) {
    if (isTooManyRequests(error)) {
        return <TooManyRequests retry={retry} />;
    }

    return (
        <div className="mx-auto max-w-md py-24 text-center">
            <h1 className="text-2xl font-semibold">Something went wrong</h1>
            <p className="mt-3 text-muted-foreground">The blog is temporarily unavailable. Please try again.</p>
            <Button className="mt-6" onClick={retry}>
                Try again
            </Button>
        </div>
    );
}
