import type { Metadata } from "next";

import { PageHeader } from "@/components/dashboard/page-header";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { SettingsForm } from "@/features/identity/components/settings-form";
import { getIdentitySettings } from "@/features/identity/queries";
import { requireAdmin } from "@/features/identity/session";

export const metadata: Metadata = { title: "Settings" };

/** Site settings, one card per bounded context that owns some. */
export default async function SettingsPage() {
    await requireAdmin();
    const identitySettings = await getIdentitySettings();

    return (
        <>
            <PageHeader title="Settings" description="Configure the site without redeploying." />

            <Card>
                <CardHeader>
                    <CardTitle>Registration</CardTitle>
                </CardHeader>
                <CardContent>
                    <SettingsForm settings={identitySettings} />
                </CardContent>
            </Card>
        </>
    );
}
