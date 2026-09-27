"use client";

import { ArrowDown, ArrowRight, Code, MapPin } from "lucide-react";
import { AnimatePresence, motion, useScroll, useSpring, useTransform } from "motion/react";
import Link from "next/link";
import { useEffect, useState } from "react";

import { profile } from "@/content/profile";

import { AnimatedCounter } from "./animated-counter";

/** Time each role stays on screen, in milliseconds. */
const ROLE_DURATION = 2600;

/** Cycles through `items` every `interval` milliseconds and returns the current index. */
function useCycle(length: number, interval: number): number {
    const [index, setIndex] = useState(0);
    useEffect(() => {
        const id = setInterval(() => setIndex((i) => (i + 1) % length), interval);
        return () => clearInterval(id);
    }, [length, interval]);
    return index;
}

/** Rotating list of roles under the name. */
function RotatingRole() {
    const index = useCycle(profile.roles.length, ROLE_DURATION);
    return (
        <span className="relative inline-flex h-[1.3em] overflow-hidden align-bottom">
            <AnimatePresence mode="popLayout" initial={false}>
                <motion.span
                    key={profile.roles[index]}
                    className="font-medium text-foreground"
                    initial={{ y: "100%", opacity: 0 }}
                    animate={{ y: 0, opacity: 1 }}
                    exit={{ y: "-100%", opacity: 0 }}
                    transition={{ duration: 0.5, ease: [0.22, 1, 0.36, 1] }}
                >
                    {profile.roles[index]}
                </motion.span>
            </AnimatePresence>
        </span>
    );
}

/** Full-screen introduction with a cursor spotlight and parallax on scroll. */
export function Hero() {
    const { scrollY } = useScroll();
    const parallax = useSpring(useTransform(scrollY, [0, 500], [0, 180]), { stiffness: 100, damping: 30 });
    const fade = useTransform(scrollY, [0, 500], [1, 0]);

    return (
        <section className="relative flex min-h-svh items-center justify-center overflow-hidden px-4 pt-24 pb-20 sm:px-6">
            <motion.div style={{ y: parallax, opacity: fade }} className="relative mx-auto max-w-6xl text-center">
                <motion.div
                    initial={{ opacity: 0, scale: 0.9 }}
                    animate={{ opacity: 1, scale: 1 }}
                    transition={{ duration: 0.6 }}
                    className="glass mx-auto mb-8 inline-flex max-w-full flex-wrap items-center justify-center gap-x-2 gap-y-1 rounded-full px-4 py-1.5 text-xs text-muted-foreground sm:text-sm"
                >
                    <span className="flex gap-2 items-center">
                        <Code className="size-4 text-brand-orange" />
                        {profile.title}
                    </span>
                    <span className="mx-1 hidden h-4 w-px bg-white/15 sm:block" />
                    <span className="flex gap-2 items-center">
                        <MapPin className="size-3.5 text-brand-amber" />
                        {profile.location}
                    </span>
                </motion.div>

                <div style={{ perspective: 800 }}>
                    <h1
                        className="text-5xl font-semibold tracking-tighter sm:text-8xl lg:text-9xl"
                        aria-label={profile.name}
                    >
                        {"Emeric "}
                        <span className="text-brand-orange">{"Fèvre"}</span>
                    </h1>
                </div>

                <motion.p
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: 0.6, duration: 0.7 }}
                    className="mx-auto mt-6 max-w-2xl text-lg sm:mt-8 text-muted-foreground sm:text-xl"
                >
                    <RotatingRole /> — {profile.tagline}
                </motion.p>

                <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: 0.85, duration: 0.7 }}
                    className="mt-10 flex flex-col items-stretch justify-center gap-3 sm:flex-row sm:flex-wrap sm:items-center sm:gap-4"
                >
                    <a
                        href="#projects"
                        className="group relative inline-flex items-center justify-center gap-2 overflow-hidden rounded-full bg-foreground px-6 py-3 font-medium text-background transition-transform hover:scale-105"
                    >
                        See my work
                        <ArrowRight className="size-4 transition-transform group-hover:translate-x-1" />
                    </a>
                    <Link
                        href="/blog"
                        className="glass inline-flex items-center justify-center gap-2 rounded-full px-6 py-3 font-medium transition-colors hover:bg-white/10"
                    >
                        Read the blog
                    </Link>
                </motion.div>

                <motion.dl
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    transition={{ delay: 1.1, duration: 1 }}
                    className="mx-auto mt-12 grid max-w-xl grid-cols-3 gap-4 sm:mt-16 sm:gap-6"
                >
                    {profile.stats.map((stat) => (
                        <div key={stat.label}>
                            <dt className="sr-only">{stat.label}</dt>
                            <dd className="text-2xl font-semibold tabular-nums sm:text-4xl">
                                <AnimatedCounter value={stat.value} suffix={stat.suffix} />
                            </dd>
                            <p className="mt-1 text-[0.65rem] tracking-wider text-muted-foreground uppercase sm:text-xs">
                                {stat.label}
                            </p>
                        </div>
                    ))}
                </motion.dl>
            </motion.div>

            <motion.a
                href="#about"
                aria-label="Scroll to the next section"
                className="absolute bottom-8 left-1/2 -translate-x-1/2 text-muted-foreground"
                animate={{ y: [0, 10, 0] }}
                transition={{ repeat: Infinity, duration: 2, ease: "easeInOut" }}
            >
                <ArrowDown className="size-5" />
            </motion.a>
        </section>
    );
}
