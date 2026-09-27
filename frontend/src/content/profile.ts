/**
 * Content of the home page.
 */
export interface Experience {
  role: string;
  company: string;
  period: string;
  kind: string;
  highlights: string[];
}

export interface Project {
  name: string;
  description: string;
  tech: string[];
  links: { label: string; href: string }[];
  logo?: string;
  status?: string;
}

export type SocialIcon = "github" | "linkedin" | "twitter";
export interface Social {
  label: string;
  icon: SocialIcon;
  href: string;
}

export interface Education {
  title: string;
  school: string;
  year: string;
}

export const profile = {
  name: "Emeric Fèvre",
  handle: "Xety",
  title: "Full-stack Developer & Designer",
  tagline:
    "Self-taught full-stack developer crafting web applications for 10+ years — from robust back-ends to polished interfaces.",
  location: "St Marcel, France",
  email: "emeric@xetaravel.com",
  cv: "https://xetaravel.com/download/Fevre_Emeric_CV.pdf",
  about: [
    "I'm a full-stack web application developer and designer, self-taught for more than ten years. I'm mostly back-end oriented — I've spent years building with PHP frameworks like Laravel and CakePHP — while staying at ease with JavaScript/TypeScript and modern front-end tooling like React and Vue - while styling with TailwindCSS, Bootstrap, LESS & SASS.",
    "I care about clean architecture, tests and automation: unit testing, Git workflows, CI/CD and DevOps are part of every project I ship. Today I'm exploring Rust to build fast and reliable back-ends — this very site runs on it with Axum, Tokio & SeaORM.",
  ],
  roles: ["Full-stack developer", "Back-end craftsman", "UI designer", "StarCitizen pilot", "Game modder"],
  stats: [
    { value: 12, suffix: "+", label: "Years of coding" },
    { value: 6, suffix: "", label: "Major projects" },
    { value: 15, suffix: "+", label: "Technologies" },
  ],
  socials: [
    { label: "GitHub", icon: "github", href: "https://github.com/Xety" },
    { label: "LinkedIn", icon: "linkedin", href: "https://www.linkedin.com/in/emeric-fevre-08a1a1261/" },
    { label: "X", icon: "twitter", href: "https://x.com/FMT_ZoRo" },
  ] as Social[],
  tools: [
    "PHP",
    "Laravel",
    "Symfony",
    "CakePHP",
    "Livewire",
    "Rust",
    "React",
    "Node.js",
    "Expo",
    "React Native",
    "Vue.js",
    "Angular",
    "MySQL",
    "PostgreSQL",
    "Redis",
    "JavaScript",
    "TypeScript",
    "Alpine.js",
    "Tailwind CSS",
    "Shadcn",
    "Bootstrap",
    "LESS",
    "SASS",
    "Photoshop",
    "Figma",
    "Docker",
  ],
  practices: ["Unit testing", "Git", "CI/CD", "DevOps", "Clean code", "MVC Architecture", "Hexagonal Architecture"],
  projects: [
    {
      name: "XetaSuite",
      logo: "/images/projects/xetasuite.svg",
      description:
        "An open-source multi-tenant ERP designed for comprehensive facility, manufacturing & production equipment, and inventory management.",
      tech: ["Laravel", "React", "Tailwind CSS", "PostgreSQL", "MCP", "Pest PHP", "i18n", "FullCalendar", "ApexCharts"],
      links: [
        { label: "GitHub", href: "https://github.com/XetaSuite/Core" },
        { label: "Docs", href: "https://xetasuite.com" },
        { label: "Demo", href: "https://demo.xetasuite.com" },
      ],
    },
    {
      name: "Selvah",
      logo: "/images/projects/selvah.png",
      description:
        "Designed, developed an ERP application for inventory management, production equipment & planning management, and user/permission control. Built with a focus on improving operational efficiency and centralizing resource management.",
      tech: ["TALL stack", "Laravel", "Alpine.js", "Livewire", "PostgreSQL", "Tailwind CSS", "FullCalendar"],
      links: [],
      status: "Confidential — acquired by my employer",
    },
    {
      name: "Xetaravel",
      logo: "/images/logo.svg",
      description:
        "My personal website with a blog, administration panel — open-source and originally built with Laravel. Now being rebuilt with Rust and Next.js.",
      tech: ["Laravel", "Livewire", "Tailwind CSS", "PostgreSQL", "Rust", "Axum", "SeaORM", "Tokio", "Next.js"],
      links: [
        { label: "GitHub (Laravel)", href: "https://github.com/XetaIO/Xetaravel" },
        { label: "GitHub (Rust)", href: "https://github.com/XetaIO/xetarust" },
      ],
    },
    {
      name: "Division Gaming",
      logo: "/images/projects/division-gaming.png",
      description:
        "Development of a website for a gaming community. Subscriptions and donations via Stripe, real-time interactive server map using Leaflet, synchronized with the game server via Redis.",
      tech: ["Laravel", "PostgreSQL", "Redis", "Leaflet", "Stripe", "Tailwind CSS", "Inertia", "React", "Python"],
      links: [{ label: "Live", href: "https://division-gaming.fr" }],
    },
    {
      name: "FrenchModdingTeam",
      logo: "/images/projects/frenchmoddingteam.png",
      description:
        "Development of a site for selling modding applications. Payment processing via Paypal. Ticketing and licensing system, management of unique HWID identifiers across the website and applications to prevent reverse engineering. A project created in 2013, which I no longer manage, with over 147,000 registered members.",
      tech: ["CakePHP", "MySQL", "Paypal", "Bootstrap", "jQuery", "i18n"],
      links: [{ label: "Live", href: "https://www.frenchmoddingteam.com" }],
    },
    {
      name: "Xeta",
      logo: "/images/projects/xeta.png",
      description: "Blog and administration panel, the ancestor of Xetaravel.",
      tech: ["CakePHP", "MySQL", "Bootstrap", "2FA", "CKEditor", "jQuery", "i18n"],
      links: [{ label: "GitHub", href: "https://github.com/XetaIO/Xeta" }],
    },
  ] as Project[],
  experiences: [
    {
      role: "Full-stack Developer & Network Administrator",
      company: "Freelance",
      period: "2015 — Present",
      kind: "Autodidact",
      highlights: [
        "PHP Backend Development: Laravel, CakePHP, API design, and complex business logic",
        "Rust Backend Development: Loco, Axum, SeaORM, Tokio, API design",
        "Frontend Development: React, Livewire, Inertia, JavaScript, TailwindCSS, Bootstrap",
        "Web Application Architecture: ERP systems, dashboards, blogs, forums, and community platforms",
        "Authentication & Authorization: users, roles, permissions, OAuth, Discord authentication",
        "Payments & Subscriptions: Stripe, PayPal, webhooks, subscriptions, donations, and licensing systems",
        "Real-Time Data & Caching: Redis, live synchronization, and real-time data display",
        "Third-Party Integrations: Discord API, RCON, payment providers, and external services",
        "Automation & Scripting: Python, data extraction, processing, and synchronization",
        "Data Management & Traceability: inventory, equipment, history tracking, QR codes, and stock movements",
        "Admin & Back-Office Development: member management, content, permissions, rewards, and configuration",
        "Large-Scale Applications: experience with platforms reaching 147,000+ registered users",
        "Long-Term Project Maintenance: building, maintaining, and evolving applications over several years",
      ],
    },
    {
      role: "Deputy Production Manager",
      company: "Selvah",
      period: "2023 — 2026",
      kind: "Permanent contract",
      highlights: [
        "Promoted after joining as a Production Line Operator in 2018",
        "Supervise and coordinate a team to ensure efficient production operations",
        "Develop and manage daily and weekly production schedules",
        "Manage scheduling, logistics and inventory",
        "Handle spare parts inventory and ensure availability of critical components",
        "Train and onboard new employees",
        "Responsible for ensuring the implementation of good hygiene practices in production",
      ],
    },
    {
      role: "Production Line Operator",
      company: "Selvah",
      period: "2018 — 2023",
      kind: "Permanent contract",
      highlights: [
        "Operated and monitored production line equipment",
        "Equipment operation and quality compliance",
        "Contributed to process improvement initiatives and supported team efficiency",
      ],
    },
    {
      role: "Full-stack Developer",
      company: "Self-employed",
      period: "2016 — Present",
      kind: "Autodidact",
      highlights: ["Design and development of personal and open-source projects"],
    },
  ] as Experience[],
  education: [
    {
      title: "Professional Certification Level 6 — Full-stack Application Developer",
      school: "OpenClassrooms",
      year: "2026",
    },
    {
      title: "Professional Certification Level 5 — Web Developer & Integrator",
      school: "OpenClassrooms",
      year: "2022",
    },
    {
      title: "Professional Certification Level 4 — Carpenter",
      school: "CFA Compagnons du Devoir, Dijon",
      year: "2007",
    },
  ] as Education[],
};
