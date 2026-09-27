import type { Metadata } from "next";

import { DashboardNav } from "@/components/dashboard/dashboard-nav";
import { SiteHeader } from "@/components/site/site-header";
import { requireAdmin } from "@/features/identity/session";

export const metadata: Metadata = { title: { default: "Dashboard", template: "%s · Dashboard" } };

/** Admin-only layout: the Rust API confirms the session and the admin role. */
export default async function DashboardLayout({ children }: LayoutProps<"/dashboard">) {
    await requireAdmin();

    return (
        <>
            <SiteHeader />
            <div className="mx-auto grid w-full max-w-7xl flex-1 content-start gap-6 px-4 py-6 sm:px-6 sm:py-10 md:grid-cols-[200px_1fr] md:gap-8">
                <aside className="sticky top-16 z-30 -mx-4 min-w-0 -mt-6 border-b border-white/5 bg-background/80 px-4 py-2 backdrop-blur-xl sm:-mx-6 sm:-mt-10 sm:px-6 md:static md:mx-0 md:mt-0 md:border-0 md:bg-transparent md:p-0 md:backdrop-blur-none">
                    <p className="mb-3 hidden px-3 text-xs tracking-[0.3em] text-muted-foreground uppercase md:block">
                        Admin
                    </p>
                    <DashboardNav />
                </aside>
                <main className="min-w-0">{children}</main>
            </div>
        </>
    );
}
