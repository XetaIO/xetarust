"use client";

import { motion, useMotionValueEvent, useScroll } from "motion/react";
import Link from "next/link";
import { useState } from "react";

import { cn } from "@/lib/utils";

const LINKS = [
  { href: "#about", label: "About" },
  { href: "#skills", label: "Skills" },
  { href: "#experience", label: "Journey" },
  { href: "#projects", label: "Projects" },
  { href: "#contact", label: "Contact" },
];

/** Floating navigation bar that turns into a glass pill once the page scrolls. */
export function HomeNav() {
  const { scrollY } = useScroll();
  const [scrolled, setScrolled] = useState(false);

  useMotionValueEvent(scrollY, "change", (y) => setScrolled(y > 24));

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
          scrolled ? "glass shadow-2xl shadow-brand-violet/10" : "border border-transparent",
        )}
      >
        <Link href="/" className="mr-2 rounded-full px-3 py-1.5 font-mono text-sm font-semibold">
          <span className="text-gradient">xeta</span>ravel
        </Link>
        <div className="hidden items-center sm:flex">
          {LINKS.map((link) => (
            <a
              key={link.href}
              href={link.href}
              className="rounded-full px-3 py-1.5 text-sm text-muted-foreground transition-colors hover:bg-white/5 hover:text-foreground"
            >
              {link.label}
            </a>
          ))}
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
