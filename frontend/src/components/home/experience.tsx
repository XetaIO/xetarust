"use client";

import { motion, useScroll, useSpring } from "motion/react";
import { useRef } from "react";
import { ArrowBigRightIcon } from "lucide-react";

import { profile } from "@/content/profile";

import { Reveal } from "./reveal";
import { SectionHeading } from "./section-heading";

/** Renders a highlight, putting its "Title:" prefix (if any) in bold. */
function Highlight({ text }: { text: string }) {
    const separator = text.indexOf(": ");
    if (separator === -1) {
        return <span>{text}</span>;
    }

    return (
        <span>
            <strong className="font-semibold text-foreground">{text.slice(0, separator)}</strong>
            {text.slice(separator)}
        </span>
    );
}

/** Vertical timeline whose line draws itself while scrolling. */
export function Experience() {
    const ref = useRef<HTMLOListElement>(null);
    const { scrollYProgress } = useScroll({ target: ref, offset: ["start 80%", "end 60%"] });
    const scaleY = useSpring(scrollYProgress, { stiffness: 100, damping: 30 });

    return (
        <section id="experience" className="mx-auto max-w-4xl scroll-mt-24 px-6 py-32">
            <SectionHeading eyebrow="Journey" title="Experience & education." />

            <ol ref={ref} className="relative space-y-10 pl-10">
                <div aria-hidden className="absolute top-2 bottom-2 left-3 w-px bg-white/10" />
                <motion.div
                    aria-hidden
                    style={{ scaleY }}
                    className="absolute top-2 bottom-2 left-3 w-px origin-top bg-brand-orange"
                />

                {profile.experiences.map((item, index) => (
                    <li key={`${item.role}-${item.period}`} className="relative">
                        <span className="absolute top-2 -left-8.5 size-3 rounded-full bg-brand-orange ring-4 ring-white/10" />
                        <Reveal delay={index * 0.05}>
                            <div className="glass rounded-2xl p-6 transition-colors hover:bg-white/[0.07]">
                                <div className="flex flex-wrap items-baseline justify-between gap-2">
                                    <h3 className="text-lg font-semibold">{item.role}</h3>
                                    <span className="font-mono text-xs text-brand-orange">{item.period}</span>
                                </div>
                                <p className="text-sm text-muted-foreground">
                                    {item.company} · {item.kind}
                                </p>
                                <ul className="mt-4 space-y-1.5 text-sm text-muted-foreground">
                                    {item.highlights.map((highlight) => (
                                        <li key={highlight} className="flex gap-2 items-baseline">
                                            <span className="text-brand-orange">
                                                <ArrowBigRightIcon className="h-3 w-3" />
                                            </span>
                                            <Highlight text={highlight} />
                                        </li>
                                    ))}
                                </ul>
                            </div>
                        </Reveal>
                    </li>
                ))}

                {profile.education.map((item) => (
                    <li key={item.title} className="relative">
                        <span className="absolute top-2 -left-8.5 size-3 rounded-full bg-brand-amber ring-4 ring-white/10" />
                        <Reveal>
                            <div className="rounded-2xl border border-dashed border-white/15 p-6">
                                <div className="flex flex-wrap items-baseline justify-between gap-2">
                                    <h3 className="font-semibold">{item.title}</h3>
                                    <span className="font-mono text-xs text-brand-amber">{item.year}</span>
                                </div>
                                <p className="text-sm text-muted-foreground">{item.school}</p>
                            </div>
                        </Reveal>
                    </li>
                ))}
            </ol>
        </section>
    );
}
