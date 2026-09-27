import Image from "next/image";

import { coverUrl } from "@/features/publishing/cover";
import { cn } from "@/lib/utils";

interface ArticleCoverProps {
    /** File name of the cover image; a subtle placeholder is shown when `null`. */
    name: string | null;
    alt: string;
    /** Responsive `sizes` hint given to `next/image`. */
    sizes: string;
    className?: string;
    /** Extra classes of the image itself (e.g. hover effects). */
    imageClassName?: string;
}

/** 16:9 cover banner of an article, or a sober branded placeholder without image. */
export function ArticleCover({ name, alt, sizes, className, imageClassName }: ArticleCoverProps) {
    return (
        <div className={cn("relative aspect-video overflow-hidden bg-card", className)}>
            {name ? (
                <Image
                    src={coverUrl(name)}
                    alt={alt}
                    fill
                    sizes={sizes}
                    className={cn("object-cover", imageClassName)}
                    loading="eager"
                />
            ) : (
                <div
                    aria-hidden
                    className="flex size-full items-center justify-center bg-linear-to-br from-brand-orange/10 via-transparent to-transparent"
                >
                    <Image src="/images/logo.svg" alt="" width={48} height={48} className="opacity-20" />
                </div>
            )}
        </div>
    );
}
