import { ArrowUpRight, type LucideIcon, Mail } from "lucide-react";

import { GithubIcon, LinkedinIcon, TwitterIcon } from "@/components/icons/brand-icons";
import { profile, type SocialIcon } from "@/content/profile";

import { CvDownload } from "./cv-download";
import { Reveal } from "./reveal";

/** Icon shown in front of each social link. */
const SOCIAL_ICONS: Record<SocialIcon, LucideIcon> = {
    github: GithubIcon,
    linkedin: LinkedinIcon,
    twitter: TwitterIcon,
};

/** Closing call-to-action with contact links. */
export function Contact() {
    return (
        <section id="contact" className="mx-auto max-w-6xl scroll-mt-24 px-4 py-20 sm:px-6 sm:py-32">
            <Reveal>
                <div className="relative overflow-hidden rounded-3xl border border-white/10 p-6 text-center sm:rounded-[2.5rem] sm:p-20">
                    <div
                        aria-hidden
                        className="animate-aurora absolute -top-1/2 left-1/2 size-160 -translate-x-1/2 rounded-full bg-brand-orange/30 blur-[100px]"
                    />
                    <div className="relative">
                        <p className="text-xs tracking-[0.3em] text-brand-orange uppercase">Contact</p>
                        <h2 className="mx-auto mt-4 max-w-3xl text-3xl font-semibold tracking-tight text-balance sm:text-6xl">
                            Let&apos;s build <span className="text-brand-orange">something great</span> together.
                        </h2>
                        <p className="mx-auto mt-6 max-w-xl text-base text-muted-foreground sm:text-lg">
                            A project, a question or just want to say hi? My inbox is always open.
                        </p>
                        <div className="mt-8 flex flex-col items-stretch justify-center gap-3 sm:mt-10 sm:flex-row sm:flex-wrap sm:items-center sm:gap-4">
                            <a
                                href={`mailto:${profile.email}`}
                                className="inline-flex max-w-full items-center justify-center gap-2 rounded-full bg-foreground px-6 py-3 font-medium text-background transition-transform hover:scale-105"
                            >
                                <Mail className="size-4 shrink-0" /> <span className="truncate">{profile.email}</span>
                            </a>
                            <CvDownload />
                        </div>
                        <ul className="mt-8 flex flex-wrap justify-center gap-x-6 gap-y-3 text-sm sm:mt-10 text-muted-foreground">
                            {profile.socials.map((social) => {
                                const Icon = SOCIAL_ICONS[social.icon];
                                return (
                                    <li key={social.href}>
                                        <a
                                            href={social.href}
                                            target="_blank"
                                            rel="noreferrer"
                                            className="group inline-flex items-center gap-2 transition-colors hover:text-foreground"
                                        >
                                            <Icon className="size-4 transition-colors group-hover:text-brand-orange" />
                                            {social.label}
                                            <ArrowUpRight className="size-3.5 transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5" />
                                        </a>
                                    </li>
                                );
                            })}
                        </ul>
                    </div>
                </div>
            </Reveal>
        </section>
    );
}
