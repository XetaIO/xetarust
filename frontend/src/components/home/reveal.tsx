"use client";

import { motion } from "motion/react";
import type { ReactNode } from "react";

interface RevealProps {
    children: ReactNode;
    /** Delay before the animation starts, in seconds. */
    delay?: number;
    /** Vertical offset the content slides from, in pixels. */
    y?: number;
    className?: string;
}

/** Fades and slides its children in the first time they enter the viewport. */
export function Reveal({ children, delay = 0, y = 24, className }: RevealProps) {
    return (
        <motion.div
            className={className}
            initial={{ opacity: 0, y, filter: "blur(6px)" }}
            whileInView={{ opacity: 1, y: 0, filter: "blur(0px)" }}
            viewport={{ once: true, margin: "-80px" }}
            transition={{ duration: 0.7, delay, ease: [0.22, 1, 0.36, 1] }}
        >
            {children}
        </motion.div>
    );
}
