"use client";

import { ArrowUpRight, Lock } from "lucide-react";
import { motion, useMotionTemplate, useMotionValue, useSpring, useTransform } from "motion/react";
import Image from "next/image";
import type { MouseEvent } from "react";

import { type Project, profile } from "@/content/profile";
import { cn } from "@/lib/utils";

import { Reveal } from "./reveal";
import { SectionHeading } from "./section-heading";

/** Maximum tilt angle of a card, in degrees. */
const MAX_TILT = 6;

/** Project card tilting in 3D and glowing under the cursor. */
function ProjectCard({ project }: { project: Project }) {
    const x = useMotionValue(0.5);
    const y = useMotionValue(0.5);
    const rotateX = useSpring(useTransform(y, [0, 1], [MAX_TILT, -MAX_TILT]), { stiffness: 200, damping: 20 });
    const rotateY = useSpring(useTransform(x, [0, 1], [-MAX_TILT, MAX_TILT]), { stiffness: 200, damping: 20 });
    const glowX = useTransform(x, (v) => `${v * 100}%`);
    const glowY = useTransform(y, (v) => `${v * 100}%`);
    const glow = useMotionTemplate`radial-gradient(400px circle at ${glowX} ${glowY}, oklch(0.68 0.19 30 / 0.25), transparent 60%)`;

    /** Stores the cursor position relative to the card (0..1). */
    function handleMove(event: MouseEvent<HTMLElement>) {
        const bounds = event.currentTarget.getBoundingClientRect();
        x.set((event.clientX - bounds.left) / bounds.width);
        y.set((event.clientY - bounds.top) / bounds.height);
    }

    /** Puts the card back flat when the cursor leaves. */
    function handleLeave() {
        x.set(0.5);
        y.set(0.5);
    }

    return (
        <motion.article
            onMouseMove={handleMove}
            onMouseLeave={handleLeave}
            style={{ rotateX, rotateY, transformPerspective: 1000 }}
            className="glass group relative flex h-full flex-col overflow-hidden rounded-3xl p-7"
        >
            <motion.div
                aria-hidden
                className="pointer-events-none absolute inset-0 opacity-0 transition-opacity duration-300 group-hover:opacity-100"
                style={{ background: glow }}
            />
            <div className="relative flex items-start justify-between gap-4">
                <div className="flex items-center gap-4">
                    {project.logo && (
                        <div
                            className={cn(
                                "relative shrink-0 overflow-hidden rounded-lg bg-white/3 shadow-lg shadow-black/20 ring-1 ring-white/10 transition-transform duration-300 group-hover:scale-105 size-14",
                            )}
                        >
                            <Image
                                src={project.logo}
                                alt={`${project.name} logo`}
                                fill
                                sizes="56px"
                                className="object-contain p-1.5"
                            />
                        </div>
                    )}
                    <h3 className={cn("font-semibold tracking-tight text-2xl")}>{project.name}</h3>
                </div>
                {project.status && (
                    <span className="inline-flex items-center gap-1 rounded-full bg-white/5 px-2.5 py-1 text-xs text-muted-foreground">
                        <Lock className="size-3" /> Private
                    </span>
                )}
            </div>
            <p className="relative mt-3 text-muted-foreground">{project.description}</p>
            {project.status && <p className="relative mt-2 text-xs text-muted-foreground/70">{project.status}</p>}

            <div className="relative mt-auto pt-6">
                <ul className="flex flex-wrap gap-2">
                    {project.tech.map((tech) => (
                        <li
                            key={tech}
                            className="rounded-md bg-white/5 px-2 py-1 font-mono text-xs text-muted-foreground"
                        >
                            {tech}
                        </li>
                    ))}
                </ul>
                {project.links.length > 0 && (
                    <div className="mt-5 flex gap-4">
                        {project.links.map((link) => (
                            <a
                                key={link.href}
                                href={link.href}
                                target="_blank"
                                rel="noreferrer"
                                className="inline-flex items-center gap-1 text-sm font-medium text-brand-orange hover:underline"
                            >
                                {link.label}
                                <ArrowUpRight className="size-3.5 transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5" />
                            </a>
                        ))}
                    </div>
                )}
            </div>
        </motion.article>
    );
}

/** Projects section. */
export function Projects() {
    return (
        <section id="projects" className="mx-auto max-w-6xl scroll-mt-24 px-6 py-32">
            <SectionHeading
                eyebrow="Projects"
                title="Things I've built."
                description="Open-source platforms, e-commerce, and business applications used every day."
            />
            <div className="grid auto-rows-fr gap-5 md:grid-cols-6">
                {profile.projects.map((project, index) => (
                    <Reveal key={project.name} delay={index * 0.08} className="md:col-span-3">
                        <ProjectCard project={project} />
                    </Reveal>
                ))}
            </div>
        </section>
    );
}
