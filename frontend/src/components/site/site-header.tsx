import { LayoutDashboard, LogOut } from "lucide-react";
import Link from "next/link";

import { Logo } from "@/components/site/logo";
import { buttonVariants } from "@/components/ui/button";
import { logout } from "@/features/identity/actions";
import { getCurrentUser } from "@/features/identity/session";

/** Header of the blog and auth pages, aware of the current session. */
export async function SiteHeader() {
    const user = await getCurrentUser();

    return (
        <header className="sticky top-0 z-40 border-b border-white/5 bg-background/70 backdrop-blur-xl">
            <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-6">
                <div className="flex items-center gap-6">
                    <Link href="/" className="font-mono font-semibold">
                        <Logo />
                    </Link>
                    <nav className="flex gap-4 text-sm text-muted-foreground">
                        <Link href="/" className="hover:text-foreground">
                            About
                        </Link>
                        <Link href="/blog" className="hover:text-foreground">
                            Blog
                        </Link>
                    </nav>
                </div>

                {user ? (
                    <div className="flex items-center gap-2">
                        <span className="hidden text-sm text-muted-foreground sm:inline">
                            Hi, <span className="text-foreground">{user.username}</span>
                        </span>
                        {user.role === "admin" && (
                            <Link href="/dashboard" className={buttonVariants({ variant: "ghost", size: "sm" })}>
                                <LayoutDashboard /> Dashboard
                            </Link>
                        )}
                        <form action={logout}>
                            <button type="submit" className={buttonVariants({ variant: "outline", size: "sm" })}>
                                <LogOut /> Log out
                            </button>
                        </form>
                    </div>
                ) : (
                    <div className="flex items-center gap-2">
                        <Link href="/login" className={buttonVariants({ variant: "ghost", size: "sm" })}>
                            Log in
                        </Link>
                        <Link href="/register" className={buttonVariants({ size: "sm" })}>
                            Sign up
                        </Link>
                    </div>
                )}
            </div>
        </header>
    );
}
