import { ShieldCheck } from "lucide-react";
import type { Metadata } from "next";

import { ConfirmAction } from "@/components/forms/confirm-action";
import { PageHeader } from "@/components/dashboard/page-header";
import { Pagination } from "@/components/site/pagination";
import { Badge } from "@/components/ui/badge";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { unbanUser } from "@/features/identity/actions";
import { BanDialog } from "@/features/identity/components/ban-dialog";
import { RoleToggle } from "@/features/identity/components/role-toggle";
import { getUsers } from "@/features/identity/queries";
import { requireAdmin } from "@/features/identity/session";
import { formatDate, parsePage } from "@/lib/format";

import { banMember } from "./actions";

export const metadata: Metadata = { title: "Users" };

/** Registered users, their roles and bans. */
export default async function UsersPage({ searchParams }: PageProps<"/dashboard/users">) {
    const me = await requireAdmin();
    const page = parsePage((await searchParams).page);
    const users = await getUsers({ page, per_page: 20 });

    return (
        <>
            <PageHeader title="Users" description={`${users.total} registered user(s)`} />
            <div className="rounded-xl border border-white/5">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>Username</TableHead>
                            <TableHead className="hidden md:table-cell">Email</TableHead>
                            <TableHead>Role</TableHead>
                            <TableHead className="hidden lg:table-cell">Joined</TableHead>
                            <TableHead className="text-right">Actions</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {users.items.map((user) => {
                            const isSelf = user.id === me.id;
                            const isBanned = user.banned_at !== null;

                            return (
                                <TableRow key={user.id}>
                                    <TableCell className="w-full max-w-0 truncate font-medium">
                                        {user.username}
                                        {isBanned && user.ban_reason && (
                                            <p className="truncate text-xs font-normal text-muted-foreground">
                                                Banned: {user.ban_reason}
                                            </p>
                                        )}
                                    </TableCell>
                                    <TableCell className="hidden text-muted-foreground md:table-cell">
                                        {user.email}
                                    </TableCell>
                                    <TableCell>
                                        <div className="flex gap-1">
                                            <Badge variant={user.role === "admin" ? "default" : "secondary"}>
                                                {user.role}
                                            </Badge>
                                            {isBanned && <Badge variant="destructive">banned</Badge>}
                                        </div>
                                    </TableCell>
                                    <TableCell className="hidden text-muted-foreground lg:table-cell">
                                        {formatDate(user.created_at)}
                                    </TableCell>
                                    <TableCell>
                                        <div className="flex justify-end gap-2">
                                            <RoleToggle
                                                userId={user.id}
                                                role={user.role}
                                                isSelf={isSelf}
                                                isBanned={isBanned}
                                            />
                                            {isBanned ? (
                                                <ConfirmAction
                                                    action={unbanUser.bind(null, user.id)}
                                                    title={`Unban ${user.username}?`}
                                                    description="They will be able to log in and comment again."
                                                    confirmLabel="Unban"
                                                >
                                                    <ShieldCheck />{" "}
                                                    <span className="sr-only sm:not-sr-only">Unban</span>
                                                </ConfirmAction>
                                            ) : (
                                                !isSelf &&
                                                user.role !== "admin" && (
                                                    <BanDialog
                                                        username={user.username}
                                                        action={banMember.bind(null, user.id)}
                                                    />
                                                )
                                            )}
                                        </div>
                                    </TableCell>
                                </TableRow>
                            );
                        })}
                    </TableBody>
                </Table>
            </div>
            <Pagination
                page={users.page}
                totalPages={users.total_pages}
                hrefFor={(target) => `/dashboard/users?page=${target}`}
            />
        </>
    );
}
