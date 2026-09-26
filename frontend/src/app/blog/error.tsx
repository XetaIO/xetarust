"use client";

import { Button } from "@/components/ui/button";

/** Error boundary of the blog (e.g. the Rust API is unreachable). */
export default function BlogError({ reset }: { error: Error & { digest?: string }; reset: () => void }) {
    return (
        <div className="mx-auto max-w-md py-24 text-center">
            <h1 className="text-2xl font-semibold">Something went wrong</h1>
            <p className="mt-3 text-muted-foreground">The blog is temporarily unavailable. Please try again.</p>
            <Button className="mt-6" onClick={reset}>
                Try again
            </Button>
        </div>
    );
}
