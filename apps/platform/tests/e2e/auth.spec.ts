import { test, expect } from "@playwright/test";
import { APP_URL, APP_URL_DASHBOARD } from "./regex";

const date = new Date();

test("subscribe", async ({ page }) => {
  await page.goto(APP_URL);
  await expect(page).toHaveURL(APP_URL + "/auth/login");

  await page.locator("[data-id=subscribe-link]").click();
  await page.locator("input#lastname").click();
  await page.locator("input#lastname").fill("Cailler");
  await page.locator("input#firstname").click();
  await page.locator("input#firstname").fill("Alexandre");
  await page.locator("input#email").click();
  await page
    .locator("input#email")
    .fill(`${date.getTime()}-login.test@france-nuage.fr`);
  await page.locator("input#password").click();
  await page.locator("input#password").fill("test1234");
  await page.locator("button[type=submit]").click();

  await page.waitForURL(APP_URL_DASHBOARD);
  await expect(page).toHaveURL(APP_URL_DASHBOARD);
});

test("login", async ({ page }) => {
  await page.goto(APP_URL);
  await expect(page).toHaveURL(APP_URL + "/auth/login");

  await page.locator("input#email").click();
  await page.locator("input#email").fill(`login.test@france-nuage.fr`);
  await page.locator("input#password").click();
  await page.locator("input#password").fill("test1234");
  await page.locator("button[type=submit]").click();

  await page.waitForURL(APP_URL_DASHBOARD);
  await expect(page).toHaveURL(APP_URL_DASHBOARD);
});

test("forgot password", async ({ page }) => {
  await page.goto(APP_URL);
  await expect(page).toHaveURL(APP_URL + "/auth/login");

  await page.locator("[data-id=forgot-password-link]").click();
  await page.locator("input#email").click();
  await page
    .locator("input#email")
    .fill(`${date.getTime()}-login.test@france-nuage.fr`);
  await page.locator("button[type=submit]").click();
});
