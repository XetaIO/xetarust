import { expect, test } from "@playwright/test";

test("the CV downloads as an attachment once the captcha is solved", async ({ page }) => {
  await page.goto("/");
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "Download my CV" }).click();
  expect((await download).suggestedFilename()).toBe("CV_Emeric_Fevre.pdf");
});

test("the CV endpoint only accepts a POST", async ({ request }) => {
  expect((await request.get("/cv")).status()).toBe(405);
});

test("the CV endpoint requires a captcha token", async ({ request }) => {
  const response = await request.post("/cv", { data: {} });
  expect(response.status()).toBe(422);
  const body = await response.json();
  expect(body.error).toBe("validation_error");
  expect(body.fields.captcha_token).toEqual(["is required"]);
});
