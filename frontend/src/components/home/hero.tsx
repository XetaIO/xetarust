"use client";

import { ArrowDown, ArrowRight, MapPin, Sparkles } from "lucide-react";
import {
  AnimatePresence,
  motion,
  useMotionTemplate,
  useMotionValue,
  useScroll,
  useSpring,
  useTransform,
} from "motion/react";
import Link from "next/link";
import { type MouseEvent, useEffect, useState } from "react";

import { profile } from "@/content/profile";

import { AnimatedCounter } from "./animated-counter";

/** Delay between two letters of the name reveal, in seconds. */
const LETTER_STAGGER = 0.045;
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

/** Name revealed letter by letter with a spring and a blur. */
function AnimatedName({ text }: { text: string }) {
  return (
    <h1 className="text-6xl font-semibold tracking-tighter sm:text-8xl lg:text-9xl" aria-label={text}>
      {text.split(" ").map((word, w) => (
        <span key={word} className="mr-[0.2em] inline-block whitespace-nowrap last:mr-0">
          {word.split("").map((letter, l) => (
            <motion.span
              key={`${letter}-${l}`}
              aria-hidden
              className={w === 1 ? "text-gradient inline-block" : "inline-block"}
              initial={{ opacity: 0, y: 80, rotateX: -90, filter: "blur(12px)" }}
              animate={{ opacity: 1, y: 0, rotateX: 0, filter: "blur(0px)" }}
              transition={{
                type: "spring",
                damping: 14,
                stiffness: 120,
                delay: 0.3 + (w * 6 + l) * LETTER_STAGGER,
              }}
            >
              {letter}
            </motion.span>
          ))}
        </span>
      ))}
    </h1>
  );
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
  const mouseX = useMotionValue(-1000);
  const mouseY = useMotionValue(-1000);
  const spotlight = useMotionTemplate`radial-gradient(600px circle at ${mouseX}px ${mouseY}px, oklch(0.62 0.24 295 / 0.15), transparent 70%)`;

  const { scrollY } = useScroll();
  const parallax = useSpring(useTransform(scrollY, [0, 600], [0, 160]), { stiffness: 80, damping: 20 });
  const fade = useTransform(scrollY, [0, 500], [1, 0]);

  /** Moves the spotlight under the cursor. */
  function handleMouseMove(event: MouseEvent<HTMLElement>) {
    const bounds = event.currentTarget.getBoundingClientRect();
    mouseX.set(event.clientX - bounds.left);
    mouseY.set(event.clientY - bounds.top);
  }

  return (
    <section
      onMouseMove={handleMouseMove}
      className="relative flex min-h-svh items-center justify-center overflow-hidden px-6 pt-24"
    >
      <motion.div aria-hidden className="pointer-events-none absolute inset-0" style={{ background: spotlight }} />

      <motion.div style={{ y: parallax, opacity: fade }} className="relative mx-auto max-w-6xl text-center">
        <motion.div
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.6 }}
          className="glass mx-auto mb-8 inline-flex items-center gap-2 rounded-full px-4 py-1.5 text-sm text-muted-foreground"
        >
          <Sparkles className="size-4 text-brand-cyan" />
          {profile.title}
          <span className="mx-1 h-4 w-px bg-white/15" />
          <MapPin className="size-3.5" />
          {profile.location}
        </motion.div>

        <div style={{ perspective: 800 }}>
          <AnimatedName text={profile.name} />
        </div>

        <motion.p
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 1.2, duration: 0.7 }}
          className="mx-auto mt-8 max-w-2xl text-lg text-muted-foreground sm:text-xl"
        >
          <RotatingRole /> — {profile.tagline}
        </motion.p>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 1.45, duration: 0.7 }}
          className="mt-10 flex flex-wrap items-center justify-center gap-4"
        >
          <a
            href="#projects"
            className="group relative inline-flex items-center gap-2 overflow-hidden rounded-full bg-foreground px-6 py-3 font-medium text-background transition-transform hover:scale-105"
          >
            See my work
            <ArrowRight className="size-4 transition-transform group-hover:translate-x-1" />
          </a>
          <Link
            href="/blog"
            className="glass inline-flex items-center gap-2 rounded-full px-6 py-3 font-medium transition-colors hover:bg-white/10"
          >
            Read the blog
          </Link>
        </motion.div>

        <motion.dl
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 1.8, duration: 0.8 }}
          className="mx-auto mt-16 grid max-w-xl grid-cols-3 gap-6"
        >
          {profile.stats.map((stat) => (
            <div key={stat.label}>
              <dt className="sr-only">{stat.label}</dt>
              <dd className="text-3xl font-semibold tabular-nums sm:text-4xl">
                <AnimatedCounter value={stat.value} suffix={stat.suffix} />
              </dd>
              <p className="mt-1 text-xs tracking-wider text-muted-foreground uppercase">{stat.label}</p>
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
