import { LayoutDashboard, LogIn, LogOut, UserPlus } from "lucide-react";
import Link from "next/link";

import { Logo } from "@/components/site/logo";
import { MobileMenu } from "@/components/site/mobile-menu";
import { buttonVariants } from "@/components/ui/button";
import { logout } from "@/features/identity/actions";
import { getIdentitySettings } from "@/features/identity/queries";
import { getCurrentUser } from "@/features/identity/session";
import { cn } from "@/lib/utils";

/** Classes of a row inside the mobile menu panel. */
const MENU_ITEM =
    "flex items-center gap-2 rounded-xl px-4 py-3 text-sm text-muted-foreground transition-colors hover:bg-white/5 hover:text-foreground";

/** Header of the blog and auth pages, aware of the current session and of whether registrations are open. */
export async function SiteHeader() {
    const [user, settings] = await Promise.all([getCurrentUser(), getIdentitySettings()]);
    const canRegister = settings.registration_enabled;

    return (
        <header className="sticky top-0 z-40 border-b border-white/5 bg-background/70 backdrop-blur-xl">
            <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-4 sm:px-6">
                <div className="flex items-center gap-6">
                    <Link href="/" className="font-semibold">
                        <Logo />
                    </Link>
                    <nav className="hidden gap-4 text-sm text-muted-foreground sm:flex">
                        <Link href="/" className="hover:text-foreground">
                            About
                        </Link>
                        <Link href="/blog" className="hover:text-foreground">
                            Blog
                        </Link>
                    </nav>
                </div>

                {user ? (
                    <div className="hidden items-center gap-2 sm:flex">
                        <span className="hidden text-sm text-muted-foreground sm:inline">
                            Hi, <span className="text-foreground">{user.username}</span>
                        </span>
                        {user.role === "admin" && (
                            <Link href="/dashboard" className={buttonVariants({ variant: "ghost" })}>
                                <LayoutDashboard /> Dashboard
                            </Link>
                        )}
                        <form action={logout}>
                            <button type="submit" className={buttonVariants({ variant: "outline" })}>
                                <LogOut /> Log out
                            </button>
                        </form>
                    </div>
                ) : (
                    <div className="hidden items-center gap-2 sm:flex">
                        <Link href="/login" className={buttonVariants({ variant: "ghost" })}>
                            Log in
                        </Link>
                        {canRegister && (
                            <Link href="/register" className={buttonVariants()}>
                                Sign up
                            </Link>
                        )}
                    </div>
                )}

                <MobileMenu>
                    <Link href="/" className={MENU_ITEM}>
                        About
                    </Link>
                    <Link href="/blog" className={MENU_ITEM}>
                        Blog
                    </Link>
                    <div className="my-1 h-px bg-white/10" />
                    {user ? (
                        <>
                            <p className="px-4 py-2 text-sm text-muted-foreground">
                                Hi, <span className="text-foreground">{user.username}</span>
                            </p>
                            {user.role === "admin" && (
                                <Link href="/dashboard" className={MENU_ITEM}>
                                    <LayoutDashboard className="size-4" /> Dashboard
                                </Link>
                            )}
                            <form action={logout}>
                                <button type="submit" className={cn(MENU_ITEM, "w-full cursor-pointer")}>
                                    <LogOut className="size-4" /> Log out
                                </button>
                            </form>
                        </>
                    ) : (
                        <>
                            <Link href="/login" className={MENU_ITEM}>
                                <LogIn className="size-4" /> Log in
                            </Link>
                            {canRegister && (
                                <Link href="/register" className={MENU_ITEM}>
                                    <UserPlus className="size-4" /> Sign up
                                </Link>
                            )}
                        </>
                    )}
                </MobileMenu>
            </div>
        </header>
    );
}
