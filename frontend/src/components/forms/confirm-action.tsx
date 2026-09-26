"use client";

import { type ReactNode, useState, useTransition } from "react";
import { toast } from "sonner";

import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
    AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import type { FormState } from "@/lib/forms";

interface ConfirmActionProps {
    /** Server Action run once the user confirms. */
    action: () => Promise<FormState>;
    title: string;
    description: string;
    /** Content of the trigger button. */
    children: ReactNode;
    confirmLabel?: string;
}

/** Destructive button asking for confirmation, then toasting the action result. */
export function ConfirmAction({ action, title, description, children, confirmLabel = "Delete" }: ConfirmActionProps) {
    const [open, setOpen] = useState(false);
    const [pending, startTransition] = useTransition();

    /** Runs the action and reports its outcome. */
    function confirm() {
        setOpen(false);
        startTransition(async () => {
            const result = await action();
            if (result?.success) {
                toast.success(result.message);
            } else {
                toast.error(result?.message ?? "Something went wrong.");
            }
        });
    }

    return (
        <AlertDialog open={open} onOpenChange={setOpen}>
            <AlertDialogTrigger render={<Button variant="destructive" size="sm" disabled={pending} />}>
                {children}
            </AlertDialogTrigger>
            <AlertDialogContent>
                <AlertDialogHeader>
                    <AlertDialogTitle>{title}</AlertDialogTitle>
                    <AlertDialogDescription>{description}</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogCancel>Cancel</AlertDialogCancel>
                    <AlertDialogAction variant="destructive" onClick={confirm}>
                        {confirmLabel}
                    </AlertDialogAction>
                </AlertDialogFooter>
            </AlertDialogContent>
        </AlertDialog>
    );
}
