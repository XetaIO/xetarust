import Link from "next/link";

/** Footer shared by every public page. */
export function SiteFooter() {
    return (
        <footer className="border-t border-white/5">
            <div className="mx-auto flex max-w-6xl flex-col items-center justify-between gap-4 px-6 py-8 text-sm text-muted-foreground sm:flex-row">
                <p>© {new Date().getFullYear()} Xetaravel. Built with Rust, Axum & Next.js.</p>
                <nav className="flex gap-6">
                    <Link href="/" className="hover:text-foreground">
                        Home
                    </Link>
                    <Link href="/blog" className="hover:text-foreground">
                        Blog
                    </Link>
                    <a
                        href="https://github.com/XetaIO/xetarust"
                        target="_blank"
                        rel="noreferrer"
                        className="hover:text-foreground"
                    >
                        GitHub
                    </a>
                </nav>
            </div>
        </footer>
    );
}
