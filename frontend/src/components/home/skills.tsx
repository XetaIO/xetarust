"use client";

import { profile } from "@/content/profile";

import { Reveal } from "./reveal";
import { SectionHeading } from "./section-heading";

/** How many times the badges are repeated inside one marquee track, so a track outgrows wide screens. */
const MARQUEE_REPEAT = 2;

/**
 * Infinite horizontal scrolling row of technology badges.
 *
 * Two identical tracks sit side by side, each at least as wide as the row; both slide
 * by exactly one track width (+ the gap), so the second track takes the place of the
 * first when the animation restarts and the loop is seamless.
 */
function Marquee({ items, reverse = false }: { items: string[]; reverse?: boolean }) {
    const badges = Array.from({ length: MARQUEE_REPEAT }, () => items).flat();

    return (
        <div className="group relative flex gap-3 overflow-hidden mask-[linear-gradient(to_right,transparent,black_10%,black_90%,transparent)] [--marquee-duration:80s]">
            {[0, 1].map((track) => (
                <div
                    key={track}
                    aria-hidden={track === 1}
                    className={`flex min-w-full shrink-0 justify-around gap-3 group-hover:paused ${
                        reverse ? "animate-marquee-reverse" : "animate-marquee"
                    }`}
                >
                    {badges.map((item, index) => (
                        <span
                            key={`${item}-${index}`}
                            className="glass rounded-full px-5 py-2 text-sm whitespace-nowrap text-muted-foreground transition-colors hover:text-foreground"
                        >
                            {item}
                        </span>
                    ))}
                </div>
            ))}
        </div>
    );
}

/** Skills section: technology marquees, levels and practices. */
export function Skills() {
    const half = Math.ceil(profile.tools.length / 2);

    return (
        <section id="skills" className="scroll-mt-24 py-20 sm:py-32">
            <div className="mx-auto max-w-6xl px-4 sm:px-6">
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

            <div className="mx-auto mt-20 grid max-w-6xl px-4 sm:px-6">
                <Reveal className="glass rounded-3xl p-6 sm:p-8">
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
