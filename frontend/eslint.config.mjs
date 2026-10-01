import { defineConfig, globalIgnores } from "eslint/config";
import nextVitals from "eslint-config-next/core-web-vitals";
import nextTs from "eslint-config-next/typescript";

/** Bounded contexts of the backend, mirrored by `src/features/*`. */
const FEATURES = ["identity", "publishing", "discussion", "resume"];

/**
 * A feature never imports another feature (nor its API types): features are
 * composed by the routes in `src/app`, like the Rust contexts are composed by
 * the `app` crate.
 */
const featureBoundaries = FEATURES.map((feature) => {
  const others = FEATURES.filter((other) => other !== feature);
  return {
    files: [`src/features/${feature}/**`],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: others.flatMap((other) => [
                `@/features/${other}`,
                `@/features/${other}/**`,
                `**/features/${other}/**`,
                `@/types/api/${other}/**`,
              ]),
              message: "A feature cannot import another feature: compose them in src/app instead.",
            },
          ],
        },
      ],
    },
  };
});

const eslintConfig = defineConfig([
  ...nextVitals,
  ...nextTs,
  ...featureBoundaries,
  // Override default ignores of eslint-config-next.
  globalIgnores([
    // Default ignores of eslint-config-next:
    ".next/**",
    "out/**",
    "build/**",
    "next-env.d.ts",
  ]),
]);

export default eslintConfig;
