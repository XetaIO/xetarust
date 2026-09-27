import { ChevronLeft, ChevronRight } from "lucide-react";
import Link from "next/link";

import { buttonVariants } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface PaginationProps {
    page: number;
    totalPages: number;
    /** Builds the URL of a given page. */
    hrefFor: (page: number) => string;
}

/** Previous / next links with the current page indicator. */
export function Pagination({ page, totalPages, hrefFor }: PaginationProps) {
    if (totalPages <= 1) {
        return null;
    }

    const link = (target: number, disabled: boolean, children: React.ReactNode) =>
        disabled ? (
            <span className={cn(buttonVariants({ variant: "outline", size: "sm" }), "pointer-events-none opacity-40")}>
                {children}
            </span>
        ) : (
            <Link href={hrefFor(target)} className={buttonVariants({ variant: "outline", size: "sm" })}>
                {children}
            </Link>
        );

    return (
        <nav aria-label="Pagination" className="mt-10 flex items-center justify-center gap-2 sm:gap-4">
            {link(
                page - 1,
                page <= 1,
                <>
                    <ChevronLeft /> <span className="sr-only sm:not-sr-only">Previous</span>
                </>,
            )}
            <span className="text-sm text-muted-foreground">
                Page {page} of {totalPages}
            </span>
            {link(
                page + 1,
                page >= totalPages,
                <>
                    <span className="sr-only sm:not-sr-only">Next</span> <ChevronRight />
                </>,
            )}
        </nav>
    );
}
