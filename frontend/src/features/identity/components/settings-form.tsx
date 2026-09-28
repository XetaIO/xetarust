"use client";

import { Save } from "lucide-react";
import { useActionState, useEffect } from "react";
import { toast } from "sonner";

import { FieldError, FormMessage } from "@/components/forms/field-error";
import { Button } from "@/components/ui/button";
import { updateIdentitySettings } from "@/features/identity/actions";
import type { FormState } from "@/lib/forms";
import type { IdentitySettingsDto } from "@/types/api/identity/IdentitySettingsDto";

/** Form of the Identity settings (opening or closing registrations). */
export function SettingsForm({ settings }: { settings: IdentitySettingsDto }) {
    const [state, action, pending] = useActionState<FormState, FormData>(updateIdentitySettings, null);

    useEffect(() => {
        if (state?.success) {
            toast.success(state.message);
        }
    }, [state]);

    return (
        <form action={action} className="space-y-4">
            <div className="space-y-1">
                <label className="flex items-center gap-3 text-sm">
                    <input
                        type="checkbox"
                        name="registration_enabled"
                        defaultChecked={settings.registration_enabled}
                        className="size-4 accent-brand-orange"
                    />
                    Allow new registrations
                </label>
                <p className="pl-7 text-xs text-muted-foreground">
                    When disabled, visitors can no longer create an account; existing members can still log in.
                </p>
                <FieldError messages={state?.fields?.registration_enabled} />
            </div>
            {!state?.success && <FormMessage message={state?.message} />}
            <Button type="submit" disabled={pending}>
                <Save /> Save
            </Button>
        </form>
    );
}
