import { ArrowUpRight, Download, Mail } from "lucide-react";

import { profile } from "@/content/profile";

import { Reveal } from "./reveal";

/** Closing call-to-action with contact links. */
export function Contact() {
  return (
    <section id="contact" className="mx-auto max-w-6xl scroll-mt-24 px-6 py-32">
      <Reveal>
        <div className="relative overflow-hidden rounded-[2.5rem] border border-white/10 p-10 text-center sm:p-20">
          <div
            aria-hidden
            className="animate-aurora absolute -top-1/2 left-1/2 size-[40rem] -translate-x-1/2 rounded-full bg-brand-violet/30 blur-[100px]"
          />
          <div className="relative">
            <p className="font-mono text-xs tracking-[0.3em] text-brand-cyan uppercase">Contact</p>
            <h2 className="mx-auto mt-4 max-w-3xl text-4xl font-semibold tracking-tight text-balance sm:text-6xl">
              Let&apos;s build <span className="text-gradient">something great</span> together.
            </h2>
            <p className="mx-auto mt-6 max-w-xl text-lg text-muted-foreground">
              A project, a question or just want to say hi? My inbox is always open.
            </p>
            <div className="mt-10 flex flex-wrap justify-center gap-4">
              <a
                href={`mailto:${profile.email}`}
                className="inline-flex items-center gap-2 rounded-full bg-foreground px-6 py-3 font-medium text-background transition-transform hover:scale-105"
              >
                <Mail className="size-4" /> {profile.email}
              </a>
              <a
                href={profile.cv}
                className="glass inline-flex items-center gap-2 rounded-full px-6 py-3 font-medium transition-colors hover:bg-white/10"
              >
                <Download className="size-4" /> Download my CV
              </a>
            </div>
            <ul className="mt-10 flex flex-wrap justify-center gap-6 text-sm text-muted-foreground">
              {profile.socials.map((social) => (
                <li key={social.href}>
                  <a
                    href={social.href}
                    target="_blank"
                    rel="noreferrer"
                    className="inline-flex items-center gap-1 hover:text-foreground"
                  >
                    {social.label} <ArrowUpRight className="size-3.5" />
                  </a>
                </li>
              ))}
            </ul>
          </div>
        </div>
      </Reveal>
    </section>
  );
}
