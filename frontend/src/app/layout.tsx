import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";

import { Toaster } from "@/components/ui/sonner";
import { profile } from "@/content/profile";

import "./globals.css";

const geistSans = Geist({
    variable: "--font-geist-sans",
    subsets: ["latin"],
});

const geistMono = Geist_Mono({
    variable: "--font-geist-mono",
    subsets: ["latin"],
});

export const metadata: Metadata = {
    title: {
        default: `${profile.name} — ${profile.title}`,
        template: `%s · Xetaravel`,
    },
    description: profile.tagline,
};

/** Root layout: fonts, dark theme and toast notifications. */
export default function RootLayout({ children }: LayoutProps<"/">) {
    return (
        <html lang="en" className={`dark ${geistSans.variable} ${geistMono.variable} h-full antialiased`}>
            <body className="flex min-h-full flex-col">
                {children}
                <Toaster theme="dark" position="bottom-right" richColors />
            </body>
        </html>
    );
}
