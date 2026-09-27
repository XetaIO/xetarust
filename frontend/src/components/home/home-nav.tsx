"use client";

import { motion, useMotionValueEvent, useScroll } from "motion/react";
import Link from "next/link";
import { useEffect, useState } from "react";

import { Logo } from "@/components/site/logo";
import { cn } from "@/lib/utils";

const LINKS = [
    { id: "about", label: "About" },
    { id: "skills", label: "Skills" },
    { id: "experience", label: "Journey" },
    { id: "projects", label: "Projects" },
    { id: "contact", label: "Contact" },
];

/** Fraction of the viewport height a section's top must cross to become the active one. */
const ACTIVATION_LINE = 0.4;

/**
 * Returns the id of the section currently being read: the last one whose top crossed
 * the activation line, or the last section once the page is scrolled to the bottom.
 */
function findActiveSection(): string | null {
    const { innerHeight, scrollY } = window;
    if (innerHeight + scrollY >= document.documentElement.scrollHeight - 2) {
        return LINKS[LINKS.length - 1].id;
    }

    let active: string | null = null;
    for (const { id } of LINKS) {
        const section = document.getElementById(id);
        if (section && section.getBoundingClientRect().top <= innerHeight * ACTIVATION_LINE) {
            active = id;
        }
    }
    return active;
}

/**
 * Floating navigation bar that turns into a glass pill once the page scrolls and
 * highlights the link of the section being read (scroll spy).
 */
export function HomeNav() {
    const { scrollY } = useScroll();
    const [scrolled, setScrolled] = useState(false);
    const [active, setActive] = useState<string | null>(null);

    useMotionValueEvent(scrollY, "change", (y) => {
        setScrolled(y > 24);
        setActive(findActiveSection());
    });

    // Sync once after the first layout, e.g. when the page is reloaded mid-scroll or opened on an anchor.
    useEffect(() => {
        const frame = requestAnimationFrame(() => setActive(findActiveSection()));
        return () => cancelAnimationFrame(frame);
    }, []);

    return (
        <motion.header
            initial={{ y: -40, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
            className="fixed inset-x-0 top-4 z-50 flex justify-center px-4"
        >
            <nav
                className={cn(
                    "flex items-center gap-1 rounded-full px-2 py-2 transition-all duration-500",
                    scrolled ? "glass shadow-2xl shadow-brand-orange/10" : "border border-transparent",
                )}
            >
                <Link href="/" className="mr-2 rounded-full px-3 py-1.5 font-mono text-sm font-semibold">
                    <Logo />
                </Link>
                <div className="hidden items-center sm:flex">
                    {LINKS.map((link) => {
                        const isActive = link.id === active;
                        return (
                            <a
                                key={link.id}
                                href={`#${link.id}`}
                                aria-current={isActive ? "location" : undefined}
                                className={cn(
                                    "relative rounded-full px-3 py-1.5 text-sm transition-colors hover:text-foreground",
                                    isActive ? "text-foreground" : "text-muted-foreground hover:bg-white/5",
                                )}
                            >
                                {isActive && (
                                    <motion.span
                                        layoutId="home-nav-active"
                                        className="absolute inset-0 rounded-full bg-brand-orange/15 ring-1 ring-brand-orange/30"
                                        transition={{ type: "spring", stiffness: 400, damping: 32 }}
                                    />
                                )}
                                <span className="relative">{link.label}</span>
                            </a>
                        );
                    })}
                </div>
                <Link
                    href="/blog"
                    className="ml-1 rounded-full bg-foreground px-4 py-1.5 text-sm font-medium text-background transition-transform hover:scale-105"
                >
                    Blog
                </Link>
            </nav>
        </motion.header>
    );
}
