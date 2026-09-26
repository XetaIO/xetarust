/**
 * Content of the home page, taken from https://xetaravel.com/@me.
 * Edit this file to update the presentation: components only render it.
 */

export interface Skill {
  name: string;
  /** Self-assessed level, from 0 to 100. */
  level: number;
}

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
  /** Featured projects take a larger tile in the bento grid. */
  featured?: boolean;
  status?: string;
}

export interface Education {
  title: string;
  school: string;
  year: string;
}

export const profile = {
  name: "Emeric Fevre",
  handle: "Xety",
  title: "Full-stack Developer & Designer",
  tagline:
    "Self-taught full-stack developer crafting web applications for 10+ years — from robust back-ends to polished interfaces.",
  location: "St Marcel, France",
  email: "emeric@xetaravel.com",
  cv: "https://xetaravel.com/download/Fevre_Emeric_CV.pdf",
  about: [
    "I'm a full-stack web application developer and designer, self-taught for more than ten years. I'm mostly back-end oriented — I've spent years building with PHP frameworks like Laravel, Symfony and CakePHP — while staying at ease with JavaScript and modern front-end tooling.",
    "I care about clean architecture, tests and automation: unit testing, Git workflows, CI/CD and DevOps are part of every project I ship. Today I'm exploring Rust to build fast and reliable back-ends — this very site runs on it.",
  ],
  roles: ["Full-stack developer", "Back-end craftsman", "UI designer", "Open-source maintainer"],
  stats: [
    { value: 10, suffix: "+", label: "Years of coding" },
    { value: 5, suffix: "", label: "Major projects" },
    { value: 15, suffix: "+", label: "Technologies" },
  ],
  socials: [
    { label: "GitHub · Xety", href: "https://github.com/Xety" },
    { label: "GitHub · XetaIO", href: "https://github.com/XetaIO" },
    { label: "Website", href: "https://xetaravel.com" },
  ],
  skills: [
    { name: "HTML", level: 100 },
    { name: "CSS", level: 90 },
    { name: "PHP", level: 90 },
    { name: "Laravel", level: 80 },
    { name: "MySQL", level: 70 },
    { name: "JavaScript", level: 60 },
  ] as Skill[],
  tools: [
    "PHP",
    "JavaScript",
    "TypeScript",
    "Rust",
    "SASS",
    "Laravel",
    "Symfony",
    "CakePHP",
    "Vue.js",
    "Angular",
    "MySQL",
    "PostgreSQL",
    "Tailwind CSS",
    "Alpine.js",
    "Livewire",
    "Bootstrap",
    "LESS",
    "Photoshop",
  ],
  practices: ["Unit testing", "Git", "CI/CD", "DevOps", "TALL stack", "Clean architecture"],
  experiences: [
    {
      role: "Deputy Production Manager",
      company: "Selvah",
      period: "2023 — Present",
      kind: "Permanent contract",
      highlights: [
        "Supervise production teams and coordinate daily operations",
        "Manage scheduling, logistics and inventory",
        "Train and onboard new employees",
      ],
    },
    {
      role: "Full-stack Developer & Network Administrator",
      company: "Division Gaming France",
      period: "2020 — 2023",
      kind: "Volunteer",
      highlights: [
        "Built the member portal with Laravel",
        "Created the donation system and the administration panels",
        "Set up CI/CD with GitHub Actions and PHPUnit test suites",
      ],
    },
    {
      role: "Production Line Operator",
      company: "Selvah",
      period: "2018 — 2023",
      kind: "Industry",
      highlights: ["Equipment operation and quality compliance"],
    },
    {
      role: "Full-stack Developer",
      company: "Self-employed",
      period: "2016 — Present",
      kind: "Autodidact",
      highlights: ["Design and development of personal and open-source projects"],
    },
  ] as Experience[],
  projects: [
    {
      name: "Xetaravel",
      description:
        "Blog, forum and administration panel — open-source and built with the Laravel framework. Now being rebuilt with Rust and Next.js.",
      tech: ["Laravel", "Livewire", "Tailwind CSS", "MySQL"],
      links: [
        { label: "GitHub", href: "https://github.com/XetaIO/Xetaravel" },
        { label: "Live", href: "https://xetaravel.com/discuss" },
      ],
      featured: true,
    },
    {
      name: "Selvah",
      description:
        "Custom web application for spare parts and equipment management, with planning, maintenance and incident tracking.",
      tech: ["TALL stack", "Laravel", "Alpine.js", "Livewire"],
      links: [],
      status: "Confidential — acquired by my employer",
      featured: true,
    },
    {
      name: "ARK Division",
      description: "Donation system with member accounts and live server status updates.",
      tech: ["Laravel", "MySQL"],
      links: [{ label: "Live", href: "https://discuss.ark-division.fr" }],
    },
    {
      name: "FrenchModdingTeam",
      description: "E-commerce platform with a secure payment API integration.",
      tech: ["CakePHP", "MySQL"],
      links: [{ label: "Live", href: "https://www.frenchmoddingteam.com" }],
    },
    {
      name: "Xeta",
      description: "Blog and administration panel, the ancestor of Xetaravel.",
      tech: ["CakePHP", "MySQL"],
      links: [{ label: "GitHub", href: "https://github.com/XetaIO/Xeta" }],
    },
  ] as Project[],
  education: [
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
