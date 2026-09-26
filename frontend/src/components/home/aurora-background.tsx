/** Animated blurred color blobs + grid, fixed behind the whole home page. */
export function AuroraBackground() {
    return (
        <div aria-hidden className="pointer-events-none fixed inset-0 -z-10 overflow-hidden">
            <div className="bg-grid absolute inset-0 mask-[radial-gradient(ellipse_at_top,black_20%,transparent_70%)]" />
            <div className="animate-aurora absolute -top-1/3 -left-1/4 size-[70vmax] rounded-full bg-brand-orange/25 blur-[120px]" />
            <div className="animate-aurora absolute top-1/4 -right-1/4 size-[55vmax] rounded-full bg-brand-amber/15 blur-[120px] [animation-delay:-6s]" />
            <div className="animate-aurora absolute -bottom-1/3 left-1/4 size-[50vmax] rounded-full bg-brand-red/15 blur-[120px] [animation-delay:-12s]" />
            <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,transparent_0%,var(--background)_80%)]" />
        </div>
    );
}
