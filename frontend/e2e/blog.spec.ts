import { execFileSync } from "node:child_process";

import { expect, type Page, test } from "@playwright/test";

/** Docker container of the dev database (see `docker-compose.yml`). */
const DB_CONTAINER = "xetaravel_postgres";

/** Promotes the account using `email` to admin directly in the dev database. */
function promoteToAdmin(email: string): void {
  execFileSync("docker", [
    "exec",
    DB_CONTAINER,
    "psql",
    "-U",
    "xetaravel",
    "-d",
    "xetaravel",
    "-v",
    "ON_ERROR_STOP=1",
    "-c",
    `UPDATE users SET role = 'admin' WHERE email = '${email}'`,
  ]);
}

/** Returns a short unique suffix for test data. */
function unique(): string {
  return Math.random().toString(36).slice(2, 8);
}

/** Registers a new account through the UI and returns its email. */
async function register(page: Page, username: string): Promise<string> {
  const email = `${username}@example.com`;
  await page.goto("/register");
  await page.getByLabel("Username").fill(username);
  await page.getByLabel("Email").fill(email);
  await page.getByLabel("Password").fill("super-secret");
  await page.getByRole("button", { name: "Create my account" }).click();
  await expect(page).toHaveURL(/\/blog$/);
  return email;
}

test("home page presents Emeric", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Things I've built." })).toBeAttached();
});

test("the dashboard is protected", async ({ page }) => {
  await page.goto("/dashboard");
  await expect(page).toHaveURL(/\/login\?next=%2Fdashboard/);
});

test("admin publishes an article and a member comments it", async ({ browser }) => {
  const id = unique();

  // --- Admin: register, get promoted, create a category and an article.
  const adminContext = await browser.newContext();
  const admin = await adminContext.newPage();
  const adminEmail = await register(admin, `admin_${id}`);
  promoteToAdmin(adminEmail);

  const [session] = await adminContext.cookies();
  expect(session.name).toBe("xetaravel_token");
  expect(session.httpOnly).toBe(true);
  expect(await admin.evaluate(() => document.cookie)).not.toContain("xetaravel_token");

  await admin.goto("/dashboard/categories");
  await admin.getByPlaceholder("Name").first().fill(`Rust ${id}`);
  await admin.getByRole("button", { name: "Add" }).click();
  await expect(admin.getByText("Category created.")).toBeVisible();

  const title = `Hello from Rust ${id}`;
  await admin.goto("/dashboard/articles/new");
  await admin.getByLabel("Category").selectOption({ label: `Rust ${id}` });
  await admin.getByLabel("Title").fill(title);
  await admin.getByLabel("Content (Markdown)").fill("# Intro\n\nWritten with **Axum**.");
  await admin.getByLabel("Publish this article").check();
  await admin.getByRole("button", { name: "Create article" }).click();
  await expect(admin).toHaveURL(/\/dashboard\/articles$/);
  await expect(admin.getByRole("cell", { name: title })).toBeVisible();

  // --- Visitor: reads the article but must log in to comment.
  const reader = await browser.newPage();
  await reader.goto("/blog");
  await reader.getByRole("link", { name: title }).click();
  await expect(reader.getByRole("heading", { name: "Intro" })).toBeVisible();
  await expect(reader.getByText("to join the discussion")).toBeVisible();

  // Share links point to the official intents with the encoded article URL.
  const encodedPath = encodeURIComponent(new URL(reader.url()).pathname);
  const shareHref = (name: string) => reader.getByRole("link", { name }).getAttribute("href");
  expect(await shareHref("Share on LinkedIn")).toContain("linkedin.com/sharing/share-offsite/?url=");
  expect(await shareHref("Share on LinkedIn")).toContain(encodedPath);
  expect(await shareHref("Share on X")).toContain("x.com/intent/post?url=");
  expect(await shareHref("Share on X")).toContain(encodedPath);
  expect(await shareHref("Share on Facebook")).toContain("facebook.com/sharer/sharer.php?u=");
  expect(await shareHref("Share on Facebook")).toContain(encodedPath);

  // The published article is listed in the sitemap.
  const sitemap = await reader.request.get("/sitemap.xml");
  expect(sitemap.status()).toBe(200);
  expect(await sitemap.text()).toContain(`${new URL(reader.url()).pathname}</loc>`);

  // --- Member: registers and comments.
  const memberContext = await browser.newContext();
  const member = await memberContext.newPage();
  await register(member, `member_${id}`);
  await member.goto(reader.url());
  await member.getByPlaceholder("Share your thoughts…").fill("Great article!");
  await member.getByRole("button", { name: "Post comment" }).click();
  await expect(member.getByText("Great article!")).toBeVisible();

  // --- Admin: closes the comments; existing ones stay visible, the form disappears.
  await admin.goto("/dashboard/articles");
  await admin.getByRole("row", { name: new RegExp(title) }).getByRole("link", { name: "Edit" }).click();
  await admin.getByLabel("Allow comments").uncheck();
  await admin.getByRole("button", { name: "Save changes" }).click();
  await expect(admin).toHaveURL(/\/dashboard\/articles$/);

  await member.reload();
  await expect(member.getByText("Great article!")).toBeVisible();
  await expect(member.getByText("Comments are closed on this article.")).toBeVisible();
  await expect(member.getByRole("button", { name: "Post comment" })).toHaveCount(0);

  // Members cannot open the dashboard: every page checks the admin role itself.
  for (const path of ["/dashboard", "/dashboard/articles/new", "/dashboard/users", "/dashboard/settings"]) {
    const response = await member.goto(path);
    expect(response?.status(), path).toBe(404);
  }
});

test("admin bans a member from the dashboard", async ({ browser }) => {
  const id = unique();

  const adminContext = await browser.newContext();
  const admin = await adminContext.newPage();
  promoteToAdmin(await register(admin, `admin_${id}`));

  const memberContext = await browser.newContext();
  const member = await memberContext.newPage();
  const memberEmail = await register(member, `member_${id}`);
  await expect(member.getByRole("button", { name: "Log out" })).toBeVisible();

  // --- Admin: bans the member with a reason and purges their comments.
  await admin.goto("/dashboard/users");
  const row = admin.getByRole("row", { name: new RegExp(`member_${id}`) });
  await row.getByRole("button", { name: "Ban" }).click();
  const dialog = admin.getByRole("alertdialog");
  await dialog.getByLabel("Reason (optional)").fill("Spam");
  await dialog.getByLabel("Also delete their comments (permanent)").check();
  await dialog.getByRole("button", { name: "Ban" }).click();
  await expect(admin.getByText("User banned. 0 comment(s) deleted.")).toBeVisible();
  await expect(row.getByText("banned", { exact: true })).toBeVisible();

  // --- Member: logged out on the next page load, and cannot log in again.
  await member.reload();
  await expect(member.getByRole("button", { name: "Log out" })).toHaveCount(0);

  await member.goto("/login");
  await member.getByLabel("Email").fill(memberEmail);
  await member.getByLabel("Password").fill("super-secret");
  await member.getByRole("button", { name: "Log in" }).click();
  await expect(member.getByText("Your account has been banned: Spam")).toBeVisible();
});
