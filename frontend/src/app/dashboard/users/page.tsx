import type { Metadata } from "next";

import { PageHeader } from "@/components/dashboard/page-header";
import { Pagination } from "@/components/site/pagination";
import { Badge } from "@/components/ui/badge";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { RoleToggle } from "@/features/identity/components/role-toggle";
import { getUsers } from "@/features/identity/queries";
import { getCurrentUser } from "@/features/identity/session";
import { formatDate, parsePage } from "@/lib/format";

export const metadata: Metadata = { title: "Users" };

/** Registered users and their roles. */
export default async function UsersPage({ searchParams }: PageProps<"/dashboard/users">) {
    const page = parsePage((await searchParams).page);
    const [users, me] = await Promise.all([getUsers({ page, per_page: 20 }), getCurrentUser()]);

    return (
        <>
            <PageHeader title="Users" description={`${users.total} registered user(s)`} />
            <div className="rounded-xl border border-white/5">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>Username</TableHead>
                            <TableHead>Email</TableHead>
                            <TableHead>Role</TableHead>
                            <TableHead>Joined</TableHead>
                            <TableHead className="text-right">Actions</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {users.items.map((user) => (
                            <TableRow key={user.id}>
                                <TableCell className="font-medium">{user.username}</TableCell>
                                <TableCell className="text-muted-foreground">{user.email}</TableCell>
                                <TableCell>
                                    <Badge variant={user.role === "admin" ? "default" : "secondary"}>{user.role}</Badge>
                                </TableCell>
                                <TableCell className="text-muted-foreground">{formatDate(user.created_at)}</TableCell>
                                <TableCell className="text-right">
                                    <RoleToggle userId={user.id} role={user.role} isSelf={user.id === me?.id} />
                                </TableCell>
                            </TableRow>
                        ))}
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
