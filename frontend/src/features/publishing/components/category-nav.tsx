import Link from "next/link";

import { cn } from "@/lib/utils";
import type { CategoryDto } from "@/types/api/publishing/CategoryDto";

/** Horizontal list of categories; `active` is the slug being browsed. */
export function CategoryNav({ categories, active }: { categories: CategoryDto[]; active?: string }) {
    const item = (href: string, label: string, isActive: boolean) => (
        <Link
            key={href}
            href={href}
            className={cn(
                "rounded-full border px-4 py-1.5 text-sm transition-colors",
                isActive
                    ? "border-brand-orange bg-brand-orange/15 text-foreground"
                    : "border-white/10 text-muted-foreground hover:border-white/25 hover:text-foreground",
            )}
        >
            {label}
        </Link>
    );

    return (
        <nav aria-label="Categories" className="flex flex-wrap gap-2">
            {item("/blog", "All", !active)}
            {categories.map((category) =>
                item(`/blog/category/${category.slug}`, category.name, category.slug === active),
            )}
        </nav>
    );
}
