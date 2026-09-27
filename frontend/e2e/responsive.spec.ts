import { expect, type Page, test } from "@playwright/test";

/** Asserts the page never scrolls horizontally at the current viewport width. */
async function expectNoHorizontalOverflow(page: Page): Promise<void> {
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  expect(overflow).toBeLessThanOrEqual(0);
}

for (const path of ["/", "/blog", "/login"]) {
  test(`${path} does not overflow horizontally on a phone`, async ({ page }) => {
    await page.goto(path);
    await expectNoHorizontalOverflow(page);
  });
}

test("the home burger menu jumps to a section and closes", async ({ page }) => {
  await page.goto("/");

  await page.getByRole("button", { name: "Open menu" }).click();
  await page.getByRole("link", { name: "Projects", exact: true }).click();

  await expect(page.getByRole("button", { name: "Open menu" })).toHaveAttribute("aria-expanded", "false");
  await expect(page.getByRole("heading", { name: "Things I've built." })).toBeInViewport();
});

test("the blog burger menu shows the session links", async ({ page }) => {
  await page.goto("/blog");

  await page.getByRole("button", { name: "Open menu" }).click();

  await expect(page.getByRole("link", { name: "Log in" })).toBeVisible();
});
