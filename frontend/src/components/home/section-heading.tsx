import { Reveal } from "./reveal";

interface SectionHeadingProps {
    /** Small uppercase label above the title. */
    eyebrow: string;
    title: string;
    description?: string;
}

/** Title block shared by every section of the home page. */
export function SectionHeading({ eyebrow, title, description }: SectionHeadingProps) {
    return (
        <Reveal className="mb-12 max-w-2xl">
            <p className="mb-3 font-mono text-xs tracking-[0.3em] text-brand-orange uppercase">{eyebrow}</p>
            <h2 className="text-4xl font-semibold tracking-tight text-balance sm:text-5xl">{title}</h2>
            {description && <p className="mt-4 text-lg text-muted-foreground">{description}</p>}
        </Reveal>
    );
}
