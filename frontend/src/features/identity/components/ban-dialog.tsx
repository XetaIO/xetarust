"use client";

import { Ban, RotateCw } from "lucide-react";
import { useActionState, useState } from "react";
import { toast } from "sonner";

import { FieldError, FormMessage } from "@/components/forms/field-error";
import {
    AlertDialog,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
    AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import type { FormState } from "@/lib/forms";

interface BanDialogProps {
    username: string;
    /**
     * Server Action banning the user, bound to its id. It also receives the
     * `delete_comments` checkbox: the route decides what "deleting comments" means.
     */
    action: (state: FormState, data: FormData) => Promise<FormState>;
}

/** Destructive button opening a form to ban a user, with an optional reason. */
export function BanDialog({ username, action }: BanDialogProps) {
    const [open, setOpen] = useState(false);
    const [state, submit, pending] = useActionState<FormState, FormData>(banAndClose, null);

    /** Runs the ban action; on success, closes the dialog and toasts the outcome. */
    async function banAndClose(previous: FormState, data: FormData): Promise<FormState> {
        const result = await action(previous, data);
        if (result?.success) {
            setOpen(false);
            toast.success(result.message);
        }
        return result;
    }

    return (
        <AlertDialog open={open} onOpenChange={setOpen}>
            <AlertDialogTrigger render={<Button variant="destructive" size="sm" />}>
                <Ban /> <span className="sr-only sm:not-sr-only">Ban</span>
            </AlertDialogTrigger>
            <AlertDialogContent>
                <form action={submit} className="grid gap-4">
                    <AlertDialogHeader>
                        <AlertDialogTitle>Ban {username}?</AlertDialogTitle>
                        <AlertDialogDescription>
                            They are logged out immediately and can no longer log in, comment or access their account
                            until you unban them.
                        </AlertDialogDescription>
                    </AlertDialogHeader>

                    <div className="grid gap-2">
                        <Label htmlFor="ban-reason">Reason (optional)</Label>
                        <Textarea
                            id="ban-reason"
                            name="reason"
                            rows={3}
                            maxLength={255}
                            placeholder="Shown to the user when they try to log in"
                            aria-invalid={Boolean(state?.fields?.reason)}
                        />
                        <FieldError messages={state?.fields?.reason} />
                    </div>

                    <label className="flex items-center gap-3 text-sm">
                        <input type="checkbox" name="delete_comments" className="size-4 accent-brand-orange" />
                        Also delete their comments (permanent)
                    </label>

                    {!state?.success && <FormMessage message={state?.message} />}

                    <AlertDialogFooter>
                        <AlertDialogCancel type="button">Cancel</AlertDialogCancel>
                        <Button type="submit" variant="destructive" disabled={pending}>
                            {pending ? (
                                <>
                                    <RotateCw className="animate-spin" /> Banning…
                                </>
                            ) : (
                                <>
                                    <Ban /> Ban
                                </>
                            )}
                        </Button>
                    </AlertDialogFooter>
                </form>
            </AlertDialogContent>
        </AlertDialog>
    );
}
