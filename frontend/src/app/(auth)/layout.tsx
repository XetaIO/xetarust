import { AuroraBackground } from "@/components/home/aurora-background";
import { SiteHeader } from "@/components/site/site-header";

/** Layout of the login and registration pages. */
export default function AuthLayout({ children }: { children: React.ReactNode }) {
    return (
        <>
            <AuroraBackground />
            <SiteHeader />
            <main className="flex flex-1 items-center justify-center px-6 py-16">{children}</main>
        </>
    );
}
