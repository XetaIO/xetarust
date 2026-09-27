"use client";

import { Turnstile, type TurnstileInstance } from "@marsidev/react-turnstile";
import Link from "next/link";
import { useActionState, useEffect, useRef, useState } from "react";

import { FieldError, FormMessage } from "@/components/forms/field-error";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { FormState } from "@/lib/forms";

interface AuthField {
    name: string;
    label: string;
    type: string;
    autoComplete: string;
}

interface AuthFormProps {
    mode: "login" | "register";
    action: (state: FormState, data: FormData) => Promise<FormState>;
    /** Page to go back to after a successful authentication. */
    next: string;
}

const FIELDS: Record<AuthFormProps["mode"], AuthField[]> = {
    login: [
        { name: "email", label: "Email", type: "email", autoComplete: "email" },
        { name: "password", label: "Password", type: "password", autoComplete: "current-password" },
    ],
    register: [
        { name: "username", label: "Username", type: "text", autoComplete: "username" },
        { name: "email", label: "Email", type: "email", autoComplete: "email" },
        { name: "password", label: "Password", type: "password", autoComplete: "new-password" },
    ],
};

const COPY = {
    login: {
        title: "Welcome back",
        description: "Log in to comment on the articles.",
        submit: "Log in",
        switchText: "No account yet?",
        switchLabel: "Sign up",
        switchHref: "/register",
    },
    register: {
        title: "Create an account",
        description: "Join the community and take part in the discussions.",
        submit: "Create my account",
        switchText: "Already registered?",
        switchLabel: "Log in",
        switchHref: "/login",
    },
};

/** Public Turnstile site key, inlined at build time; absent = captcha disabled. */
const TURNSTILE_SITE_KEY = process.env.NEXT_PUBLIC_TURNSTILE_SITE_KEY;

/** Name of the field holding the captcha token (the one Turnstile injects). */
const CAPTCHA_FIELD = "cf-turnstile-response";

/**
 * Token sent when the captcha is disabled (dev, e2e): the API then runs
 * without a Turnstile secret and accepts any token, but still requires one.
 */
const CAPTCHA_DISABLED_TOKEN = "captcha-disabled";

/** Login or registration form bound to its Server Action, protected by Turnstile. */
export function AuthForm({ mode, action, next }: AuthFormProps) {
    const [state, formAction, pending] = useActionState(action, null);
    const [token, setToken] = useState<string | null>(null);
    const [answeredState, setAnsweredState] = useState(state);
    const captcha = useRef<TurnstileInstance>(undefined);
    const copy = COPY[mode];

    // A Turnstile token can only be used once: forget it after every answer…
    if (state !== answeredState) {
        setAnsweredState(state);
        setToken(null);
    }

    // …and ask the widget for a fresh one.
    useEffect(() => {
        if (state) {
            captcha.current?.reset();
        }
    }, [state]);

    const waitingForCaptcha = Boolean(TURNSTILE_SITE_KEY) && !token;

    return (
        <Card className="glass w-full max-w-md">
            <CardHeader>
                <CardTitle className="text-2xl">{copy.title}</CardTitle>
                <CardDescription>{copy.description}</CardDescription>
            </CardHeader>
            <form action={formAction}>
                <CardContent className="space-y-4">
                    <input type="hidden" name="next" value={next} />
                    <FormMessage message={state?.message} />
                    {FIELDS[mode].map((field) => (
                        <div key={field.name} className="space-y-2">
                            <Label htmlFor={field.name}>{field.label}</Label>
                            <Input
                                id={field.name}
                                name={field.name}
                                type={field.type}
                                autoComplete={field.autoComplete}
                                required
                                aria-invalid={Boolean(state?.fields?.[field.name])}
                            />
                            <FieldError messages={state?.fields?.[field.name]} />
                        </div>
                    ))}
                    <div className="space-y-2">
                        {TURNSTILE_SITE_KEY ? (
                            <Turnstile
                                ref={captcha}
                                siteKey={TURNSTILE_SITE_KEY}
                                options={{ theme: "dark", size: "flexible" }}
                                onSuccess={setToken}
                                onExpire={() => setToken(null)}
                                onError={() => setToken(null)}
                            />
                        ) : (
                            <input type="hidden" name={CAPTCHA_FIELD} value={CAPTCHA_DISABLED_TOKEN} />
                        )}
                        <FieldError messages={state?.fields?.captcha_token} />
                    </div>
                </CardContent>
                <CardFooter className="mt-6 flex flex-col gap-4">
                    <Button type="submit" className="w-full" size="lg" disabled={pending || waitingForCaptcha}>
                        {pending ? "Please wait…" : copy.submit}
                    </Button>
                    <p className="text-sm text-muted-foreground">
                        {copy.switchText}{" "}
                        <Link
                            href={`${copy.switchHref}?next=${encodeURIComponent(next)}`}
                            className="text-brand-amber hover:underline"
                        >
                            {copy.switchLabel}
                        </Link>
                    </p>
                </CardFooter>
            </form>
        </Card>
    );
}
