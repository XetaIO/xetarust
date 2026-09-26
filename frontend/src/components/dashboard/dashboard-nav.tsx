"use client";

import { FileText, FolderTree, LayoutDashboard, Users } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";

import { cn } from "@/lib/utils";

const LINKS = [
    { href: "/dashboard", label: "Overview", icon: LayoutDashboard, exact: true },
    { href: "/dashboard/articles", label: "Articles", icon: FileText },
    { href: "/dashboard/categories", label: "Categories", icon: FolderTree },
    { href: "/dashboard/users", label: "Users", icon: Users },
];

/** Sidebar navigation of the dashboard, highlighting the current section. */
export function DashboardNav() {
    const pathname = usePathname();

    return (
        <nav className="flex gap-1 overflow-x-auto md:flex-col">
            {LINKS.map(({ href, label, icon: Icon, exact }) => {
                const active = exact ? pathname === href : pathname.startsWith(href);
                return (
                    <Link
                        key={href}
                        href={href}
                        className={cn(
                            "flex items-center gap-3 rounded-lg px-3 py-2 text-sm whitespace-nowrap transition-colors",
                            active
                                ? "bg-brand-orange/15 text-foreground"
                                : "text-muted-foreground hover:bg-white/5 hover:text-foreground",
                        )}
                    >
                        <Icon className="size-4" />
                        {label}
                    </Link>
                );
            })}
        </nav>
    );
}
