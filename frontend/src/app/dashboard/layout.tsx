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
            <div className="mx-auto grid w-full max-w-7xl flex-1 gap-8 px-6 py-10 md:grid-cols-[200px_1fr]">
                <aside>
                    <p className="mb-3 px-3 font-mono text-xs tracking-[0.3em] text-muted-foreground uppercase">
                        Admin
                    </p>
                    <DashboardNav />
                </aside>
                <main className="min-w-0">{children}</main>
            </div>
        </>
    );
}
