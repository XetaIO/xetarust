import type { LucideIcon } from "lucide-react";

import { FacebookIcon, LinkedinIcon, TwitterIcon } from "@/components/icons/brand-icons";

type ShareButtonsProps = {
    url: string;
    title: string;
};

type ShareTarget = {
    label: string;
    icon: LucideIcon;
    href: (url: string, title: string) => string;
};

const SHARE_TARGETS: ShareTarget[] = [
    {
        label: "X",
        icon: TwitterIcon,
        href: (url, title) =>
            `https://x.com/intent/post?url=${encodeURIComponent(url)}&text=${encodeURIComponent(title)}`,
    },
    {
        label: "Facebook",
        icon: FacebookIcon,
        href: (url) => `https://www.facebook.com/sharer/sharer.php?u=${encodeURIComponent(url)}`,
    },
    {
        label: "LinkedIn",
        icon: LinkedinIcon,
        href: (url) => `https://www.linkedin.com/sharing/share-offsite/?url=${encodeURIComponent(url)}`,
    },
];

/** Discreet icon links sharing an article on X, Facebook and LinkedIn through their official intents. */
export function ShareButtons({ url, title }: ShareButtonsProps) {
    return (
        <span className="inline-flex items-center gap-1">
            <span className="mr-1 text-xs text-muted-foreground">Share</span>
            {SHARE_TARGETS.map(({ label, icon: Icon, href }) => (
                <a
                    key={label}
                    href={href(url, title)}
                    target="_blank"
                    rel="noopener noreferrer"
                    aria-label={`Share on ${label}`}
                    className="rounded-full p-1.5 text-muted-foreground transition-colors hover:text-brand-orange"
                >
                    <Icon className="size-4" />
                </a>
            ))}
        </span>
    );
}
