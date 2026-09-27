import Link from "next/link";

import { buttonVariants } from "@/components/ui/button";

/** Global 404 page. */
export default function NotFound() {
    return (
        <main className="flex flex-1 flex-col items-center justify-center px-6 py-32 text-center">
            <p className="text-brand-orange text-8xl font-bold">404</p>
            <h1 className="mt-4 text-2xl font-semibold">This page does not exist.</h1>
            <p className="mt-2 text-muted-foreground">It may have been moved, or never written at all.</p>
            <Link href="/" className={buttonVariants({ className: "mt-8" })}>
                Back home
            </Link>
        </main>
    );
}
