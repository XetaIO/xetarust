import { About } from "@/components/home/about";
import { AuroraBackground } from "@/components/home/aurora-background";
import { Contact } from "@/components/home/contact";
import { Experience } from "@/components/home/experience";
import { Hero } from "@/components/home/hero";
import { HomeNav } from "@/components/home/home-nav";
import { Projects } from "@/components/home/projects";
import { Skills } from "@/components/home/skills";
import { SiteFooter } from "@/components/site/site-footer";

/** Home page: animated presentation of Emeric and his projects (fully static). */
export default function HomePage() {
    return (
        <>
            <AuroraBackground />
            <HomeNav />
            <main className="flex-1">
                <Hero />
                <About />
                <Skills />
                <Experience />
                <Projects />
                <Contact />
            </main>
            <SiteFooter />
        </>
    );
}
