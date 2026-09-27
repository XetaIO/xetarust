import type { ReactNode } from "react";

/** Title row of a dashboard page with optional actions on the right. */
export function PageHeader({
    title,
    description,
    actions,
}: {
    title: string;
    description?: string;
    actions?: ReactNode;
}) {
    return (
        <div className="mb-6 flex flex-wrap items-end justify-between gap-4 sm:mb-8">
            <div className="min-w-0">
                <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">{title}</h1>
                {description && <p className="mt-1 text-muted-foreground">{description}</p>}
            </div>
            {actions && <div className="w-full *:w-full sm:w-auto sm:*:w-auto">{actions}</div>}
        </div>
    );
}
