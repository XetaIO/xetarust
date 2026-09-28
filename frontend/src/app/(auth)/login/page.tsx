import type { Metadata } from "next";
import { redirect } from "next/navigation";

import { login } from "@/features/identity/actions";
import { AuthForm } from "@/features/identity/components/auth-form";
import { getIdentitySettings } from "@/features/identity/queries";
import { safeRedirectTarget } from "@/features/identity/redirect";
import { getCurrentUser } from "@/features/identity/session";

export const metadata: Metadata = { title: "Log in" };

/** Login page; already authenticated users are sent back to the blog. */
export default async function LoginPage({ searchParams }: PageProps<"/login">) {
    const target = safeRedirectTarget((await searchParams).next);
    if (await getCurrentUser()) {
        redirect(target);
    }

    const { registration_enabled } = await getIdentitySettings();

    return <AuthForm mode="login" action={login} next={target} canRegister={registration_enabled} />;
}
