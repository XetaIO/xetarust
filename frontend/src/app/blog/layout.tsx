import { SiteFooter } from "@/components/site/site-footer";
import { SiteHeader } from "@/components/site/site-header";

/** Layout of the public blog. */
export default function BlogLayout({ children }: LayoutProps<"/blog">) {
    return (
        <>
            <SiteHeader />
            <main className="mx-auto w-full max-w-6xl flex-1 px-6 py-12">{children}</main>
            <SiteFooter />
        </>
    );
}
