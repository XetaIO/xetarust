import { expect, test } from "@playwright/test";

test("the CV downloads as an attachment once the captcha is solved", async ({ page }) => {
  await page.goto("/");
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "Download my CV" }).click();
  expect((await download).suggestedFilename()).toBe("CV_Emeric_Fevre.pdf");
});
