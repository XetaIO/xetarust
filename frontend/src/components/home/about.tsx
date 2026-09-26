import { Code2, Rocket, ShieldCheck } from "lucide-react";

import { profile } from "@/content/profile";

import { Reveal } from "./reveal";
import { SectionHeading } from "./section-heading";

const PILLARS = [
    { icon: Code2, title: "Clean code", text: "Readable, tested and documented code built on SOLID principles." },
    { icon: ShieldCheck, title: "Reliability", text: "Automated tests and CI/CD pipelines on every project I ship." },
    { icon: Rocket, title: "Performance", text: "Fast back-ends and smooth interfaces, from Laravel to Rust." },
];

/** "About me" section: biography and working principles. */
export function About() {
    return (
        <section id="about" className="mx-auto max-w-6xl scroll-mt-24 px-6 py-32">
            <SectionHeading eyebrow="About me" title="Building the web, one well-crafted layer at a time." />
            <div className="grid gap-12 lg:grid-cols-[1.2fr_1fr]">
                <div className="space-y-6 text-lg leading-relaxed text-muted-foreground">
                    {profile.about.map((paragraph, index) => (
                        <Reveal key={paragraph} delay={index * 0.1}>
                            <p>{paragraph}</p>
                        </Reveal>
                    ))}
                </div>
                <div className="grid gap-4">
                    {PILLARS.map((pillar, index) => (
                        <Reveal key={pillar.title} delay={0.15 + index * 0.1}>
                            <div className="glass group flex gap-4 rounded-2xl p-5 transition-colors hover:bg-white/[0.07]">
                                <div className="flex size-11 shrink-0 items-center justify-center rounded-xl bg-linear-to-br from-brand-orange/30 to-brand-amber/20 transition-transform group-hover:scale-110 group-hover:rotate-6">
                                    <pillar.icon className="size-5" />
                                </div>
                                <div>
                                    <h3 className="font-medium">{pillar.title}</h3>
                                    <p className="mt-1 text-sm text-muted-foreground">{pillar.text}</p>
                                </div>
                            </div>
                        </Reveal>
                    ))}
                </div>
            </div>
        </section>
    );
}
