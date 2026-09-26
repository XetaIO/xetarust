import { MarkdownAsync } from "react-markdown";
import rehypePrettyCode from "rehype-pretty-code";
import remarkGfm from "remark-gfm";

import { cn } from "@/lib/utils";

/**
 * Renders Markdown on the server with GitHub-flavoured syntax and Shiki
 * code highlighting. Raw HTML in the source is ignored (safe by default).
 */
export async function Markdown({ source, className }: { source: string; className?: string }) {
    return (
        <div
            className={cn(
                "prose prose-invert max-w-none prose-headings:tracking-tight prose-a:text-brand-amber prose-pre:border prose-pre:border-white/10 prose-pre:bg-card prose-code:before:content-none prose-code:after:content-none",
                className,
            )}
        >
            <MarkdownAsync
                remarkPlugins={[remarkGfm]}
                rehypePlugins={[[rehypePrettyCode, { theme: "github-dark-dimmed", keepBackground: false }]]}
            >
                {source}
            </MarkdownAsync>
        </div>
    );
}
