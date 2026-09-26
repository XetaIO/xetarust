import { defineConfig, devices } from "@playwright/test";

/**
 * End-to-end tests. They expect the Rust API on :8080 (`cargo run -p xetaravel-app`)
 * and start the Next.js dev server automatically when it is not already running.
 */
export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  workers: 1,
  reporter: "list",
  use: {
    baseURL: "http://localhost:3000",
    trace: "retain-on-failure",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: {
    command: "npm run dev",
    url: "http://localhost:3000",
    reuseExistingServer: true,
    timeout: 120_000,
  },
});
