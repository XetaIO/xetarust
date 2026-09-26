"use client";

import { useTransition } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { changeUserRole } from "@/features/identity/actions";
import type { RoleDto } from "@/types/api/identity/RoleDto";

interface RoleToggleProps {
    userId: string;
    role: RoleDto;
    /** True for the current admin, who cannot demote themselves. */
    isSelf: boolean;
}

/** Button promoting a member to admin, or demoting an admin to member. */
export function RoleToggle({ userId, role, isSelf }: RoleToggleProps) {
    const [pending, startTransition] = useTransition();
    const target: RoleDto = role === "admin" ? "member" : "admin";

    /** Calls the Server Action and reports the outcome. */
    function toggle() {
        startTransition(async () => {
            const result = await changeUserRole(userId, target);
            if (result?.success) {
                toast.success(result.message);
            } else {
                toast.error(result?.message ?? "Could not change the role.");
            }
        });
    }

    return (
        <Button
            variant={role === "admin" ? "outline" : "default"}
            size="sm"
            disabled={pending || isSelf}
            onClick={toggle}
            title={isSelf ? "You cannot demote yourself" : undefined}
        >
            {role === "admin" ? "Demote to member" : "Promote to admin"}
        </Button>
    );
}
