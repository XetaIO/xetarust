import { LogIn } from "lucide-react";
import type { Metadata } from "next";
import Link from "next/link";
import { redirect } from "next/navigation";

import { buttonVariants } from "@/components/ui/button";
import { Card, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { register } from "@/features/identity/actions";
import { AuthForm } from "@/features/identity/components/auth-form";
import { getIdentitySettings } from "@/features/identity/queries";
import { safeRedirectTarget } from "@/features/identity/redirect";
import { getCurrentUser } from "@/features/identity/session";

export const metadata: Metadata = { title: "Sign up" };

/**
 * Registration page; already authenticated users are sent back to the blog.
 * Shows a notice instead of the form when an admin closed registrations.
 */
export default async function RegisterPage({ searchParams }: PageProps<"/register">) {
    const target = safeRedirectTarget((await searchParams).next);
    if (await getCurrentUser()) {
        redirect(target);
    }

    const { registration_enabled } = await getIdentitySettings();
    if (!registration_enabled) {
        return (
            <Card className="glass w-full max-w-md">
                <CardHeader>
                    <CardTitle className="text-2xl">Registrations are closed</CardTitle>
                    <CardDescription>
                        New accounts cannot be created at the moment. Existing members can still log in.
                    </CardDescription>
                </CardHeader>
                <CardFooter className="mt-6">
                    <Link
                        href={`/login?next=${encodeURIComponent(target)}`}
                        className={buttonVariants({ size: "lg", className: "w-full" })}
                    >
                        <LogIn /> Log in
                    </Link>
                </CardFooter>
            </Card>
        );
    }

    return <AuthForm mode="register" action={register} next={target} />;
}
