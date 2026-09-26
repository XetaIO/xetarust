/** Displays the validation messages returned by the API for one field. */
export function FieldError({ messages }: { messages?: string[] }) {
    if (!messages?.length) {
        return null;
    }
    return <p className="text-sm text-destructive">{messages.join(", ")}</p>;
}

/** Displays the global message of a form state, styled by outcome. */
export function FormMessage({ message, success }: { message?: string; success?: boolean }) {
    if (!message) {
        return null;
    }
    return (
        <p
            role={success ? "status" : "alert"}
            className={
                success
                    ? "rounded-lg bg-emerald-500/10 px-3 py-2 text-sm text-emerald-400"
                    : "rounded-lg bg-destructive/10 px-3 py-2 text-sm text-destructive"
            }
        >
            {message}
        </p>
    );
}
