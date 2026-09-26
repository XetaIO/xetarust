import type { Metadata } from "next";
import { redirect } from "next/navigation";

import { register } from "@/features/identity/actions";
import { AuthForm } from "@/features/identity/components/auth-form";
import { safeRedirectTarget } from "@/features/identity/redirect";
import { getCurrentUser } from "@/features/identity/session";

export const metadata: Metadata = { title: "Sign up" };

/** Registration page; already authenticated users are sent back to the blog. */
export default async function RegisterPage({ searchParams }: PageProps<"/register">) {
    const target = safeRedirectTarget((await searchParams).next);
    if (await getCurrentUser()) {
        redirect(target);
    }

    return <AuthForm mode="register" action={register} next={target} />;
}
