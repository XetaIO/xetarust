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
                "prose prose-sm prose-invert max-w-none sm:prose-base prose-headings:tracking-tight prose-a:break-words prose-a:text-brand-orange prose-a:decoration-brand-orange/50 hover:prose-a:decoration-brand-orange prose-pre:overflow-x-auto prose-pre:border prose-pre:border-white/10 prose-pre:bg-card prose-code:before:content-none prose-code:after:content-none prose-img:rounded-xl prose-table:block prose-table:overflow-x-auto",
                className,
            )}
        >
            <MarkdownAsync
                remarkPlugins={[remarkGfm]}
                rehypePlugins={[[rehypePrettyCode, { theme: "one-dark-pro", keepBackground: false }]]}
            >
                {source}
            </MarkdownAsync>
        </div>
    );
}
