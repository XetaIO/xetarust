"use client";

import { Menu, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { usePathname } from "next/navigation";
import { type MouseEvent, type ReactNode, useEffect, useId, useRef, useState } from "react";

import { buttonVariants } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface MobileMenuProps {
    /** Content of the dropdown panel (links, forms…). */
    children: ReactNode;
    /** Extra classes for the panel. */
    className?: string;
}

/**
 * Burger button (below the `sm` breakpoint only) toggling a glass dropdown panel.
 *
 * The panel is positioned against the closest positioned ancestor (the header) and
 * closes on link click, `Escape`, route change or a click outside the menu.
 */
export function MobileMenu({ children, className }: MobileMenuProps) {
    const [open, setOpen] = useState(false);
    const [openedOn, setOpenedOn] = useState<string | null>(null);
    const pathname = usePathname();
    const panelId = useId();
    const rootRef = useRef<HTMLDivElement>(null);

    // Close after navigating to another page (derived during render, no effect needed).
    if (open && openedOn !== pathname) {
        setOpen(false);
    }

    useEffect(() => {
        if (!open) {
            return;
        }

        /** Closes the menu when `Escape` is pressed. */
        function handleKey(event: KeyboardEvent) {
            if (event.key === "Escape") {
                setOpen(false);
            }
        }

        /** Closes the menu when the pointer goes down outside of it. */
        function handlePointer(event: PointerEvent) {
            if (!rootRef.current?.contains(event.target as Node)) {
                setOpen(false);
            }
        }

        document.addEventListener("keydown", handleKey);
        document.addEventListener("pointerdown", handlePointer);
        return () => {
            document.removeEventListener("keydown", handleKey);
            document.removeEventListener("pointerdown", handlePointer);
        };
    }, [open]);

    /** Toggles the panel, remembering the page it was opened on. */
    function toggle() {
        setOpenedOn(pathname);
        setOpen((value) => !value);
    }

    /** Closes the panel when one of its links is followed (including same-page anchors). */
    function handlePanelClick(event: MouseEvent<HTMLDivElement>) {
        if ((event.target as HTMLElement).closest("a")) {
            setOpen(false);
        }
    }

    return (
        <div ref={rootRef} className="sm:hidden">
            <button
                type="button"
                onClick={toggle}
                aria-expanded={open}
                aria-controls={panelId}
                aria-label={open ? "Close menu" : "Open menu"}
                className={cn(buttonVariants({ variant: "ghost", size: "icon" }), "rounded-full")}
            >
                {open ? <X /> : <Menu />}
            </button>
            <AnimatePresence>
                {open && (
                    <motion.div
                        id={panelId}
                        initial={{ opacity: 0, y: -8 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: -8 }}
                        transition={{ duration: 0.25, ease: [0.22, 1, 0.36, 1] }}
                        onClick={handlePanelClick}
                        className={cn(
                            "absolute inset-x-4 top-full mt-2 flex flex-col gap-1 rounded-2xl border border-white/10 bg-background p-2 shadow-2xl shadow-black/40",
                            className,
                        )}
                    >
                        {children}
                    </motion.div>
                )}
            </AnimatePresence>
        </div>
    );
}
