"use client";

import { motion } from "motion/react";

import { profile } from "@/content/profile";

import { Reveal } from "./reveal";
import { SectionHeading } from "./section-heading";

/** Infinite horizontal scrolling row of technology badges. */
function Marquee({ items, reverse = false }: { items: string[]; reverse?: boolean }) {
    return (
        <div className="group relative flex overflow-hidden mask-[linear-gradient(to_right,transparent,black_10%,black_90%,transparent)]">
            <div
                className={`flex shrink-0 gap-3 pr-3 group-hover:paused ${
                    reverse ? "animate-marquee-reverse" : "animate-marquee"
                }`}
            >
                {[...items, ...items].map((item, index) => (
                    <span
                        key={`${item}-${index}`}
                        className="glass rounded-full px-5 py-2 font-mono text-sm whitespace-nowrap text-muted-foreground transition-colors hover:text-foreground"
                    >
                        {item}
                    </span>
                ))}
            </div>
        </div>
    );
}

/** Skill level bar filling up when it scrolls into view. */
function SkillBar({ name, level, index }: { name: string; level: number; index: number }) {
    return (
        <div>
            <div className="mb-2 flex justify-between text-sm">
                <span className="font-medium">{name}</span>
                <span className="font-mono text-muted-foreground">{level}%</span>
            </div>
            <div className="h-2 overflow-hidden rounded-full bg-white/5">
                <motion.div
                    className="h-full rounded-full bg-linear-to-r from-brand-orange via-brand-amber to-brand-red"
                    initial={{ width: 0 }}
                    whileInView={{ width: `${level}%` }}
                    viewport={{ once: true }}
                    transition={{ duration: 1.2, delay: index * 0.1, ease: [0.22, 1, 0.36, 1] }}
                />
            </div>
        </div>
    );
}

/** Skills section: technology marquees, levels and practices. */
export function Skills() {
    const half = Math.ceil(profile.tools.length / 2);

    return (
        <section id="skills" className="scroll-mt-24 py-32">
            <div className="mx-auto max-w-6xl px-6">
                <SectionHeading
                    eyebrow="Skills"
                    title="A versatile toolbox."
                    description="From PHP frameworks to Rust, the technologies I use to bring ideas to life."
                />
            </div>

            <div className="space-y-4">
                <Marquee items={profile.tools.slice(0, half)} />
                <Marquee items={profile.tools.slice(half)} reverse />
            </div>

            <div className="mx-auto mt-20 grid max-w-6xl gap-12 px-6 lg:grid-cols-2">
                <div className="space-y-6">
                    {profile.skills.map((skill, index) => (
                        <SkillBar key={skill.name} {...skill} index={index} />
                    ))}
                </div>
                <Reveal className="glass rounded-3xl p-8">
                    <h3 className="text-xl font-semibold">Engineering practices</h3>
                    <p className="mt-2 text-muted-foreground">What makes a project last beyond its first release.</p>
                    <ul className="mt-6 flex flex-wrap gap-2">
                        {profile.practices.map((practice) => (
                            <li
                                key={practice}
                                className="rounded-lg border border-brand-orange/30 bg-brand-orange/10 px-3 py-1.5 text-sm"
                            >
                                {practice}
                            </li>
                        ))}
                    </ul>
                </Reveal>
            </div>
        </section>
    );
}
