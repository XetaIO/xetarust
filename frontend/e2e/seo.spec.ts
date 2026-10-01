import { expect, test } from "@playwright/test";

test("robots.txt hides private pages and points to the sitemap", async ({ request }) => {
  const response = await request.get("/robots.txt");
  expect(response.status()).toBe(200);

  const body = await response.text();
  expect(body).toContain("Disallow: /dashboard/");
  expect(body).toMatch(/Sitemap: https?:\/\/\S+\/sitemap\.xml/);
});

test("sitemap.xml lists the blog with absolute URLs", async ({ request }) => {
  const response = await request.get("/sitemap.xml");
  expect(response.status()).toBe(200);
  expect(response.headers()["content-type"]).toContain("xml");

  const body = await response.text();
  expect(body).toMatch(/<loc>https?:\/\/[^<]+\/blog<\/loc>/);
  expect(body).not.toContain("/dashboard");
});
