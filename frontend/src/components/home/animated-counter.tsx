"use client";

import { animate, useInView } from "motion/react";
import { useEffect, useRef } from "react";

interface AnimatedCounterProps {
    value: number;
    suffix?: string;
}

/** Counts from 0 to `value` when it scrolls into view. */
export function AnimatedCounter({ value, suffix = "" }: AnimatedCounterProps) {
    const ref = useRef<HTMLSpanElement>(null);
    const inView = useInView(ref, { once: true });

    useEffect(() => {
        if (!inView || !ref.current) {
            return;
        }
        const node = ref.current;
        const controls = animate(0, value, {
            duration: 1.6,
            ease: "easeOut",
            onUpdate: (latest) => {
                node.textContent = `${Math.round(latest)}${suffix}`;
            },
        });
        return () => controls.stop();
    }, [inView, value, suffix]);

    return <span ref={ref}>0{suffix}</span>;
}
