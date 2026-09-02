#!/usr/bin/env node
// Drive the console in Chromium; Playwright records the viewport.

import { chromium } from "playwright";
import fs from "node:fs";
import path from "node:path";

const BASE = process.env.CONSOLE_URL || "http://127.0.0.1:3040";
const DIR = process.env.VIDEO_DIR || path.resolve("target/demo");
const HOLD = Number(process.env.HOLD_MS || 2400);
const HOLD_LATER = Number(process.env.HOLD_LATER_MS || 3800);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

fs.mkdirSync(DIR, { recursive: true });

const browser = await chromium.launch({
  headless: process.env.HEADLESS === "1",
  args: ["--window-size=1280,720"],
});
const context = await browser.newContext({
  viewport: { width: 1280, height: 720 },
  recordVideo: { dir: DIR, size: { width: 1280, height: 720 } },
});
const page = await context.newPage();

async function click(testId) {
  await page.getByTestId(testId).click();
}

async function go(testId, pageId, hold = HOLD) {
  await click(testId);
  await page.getByTestId(pageId).waitFor({ timeout: 20_000 });
  await sleep(hold);
}

try {
  await page.goto(BASE + "/");
  await page.getByTestId("gate").waitFor();
  await page.getByTestId("lab-logins").waitFor({ timeout: 20_000 });
  await sleep(HOLD);
  await page.getByTestId("lab-logins").locator("tr.pick").first().click();
  await sleep(HOLD);
  await page.getByTestId("email").click();
  await sleep(400);
  await page.getByTestId("password").click();
  await sleep(HOLD);
  await click("submit");
  await page.getByTestId("page-home").waitFor({ timeout: 20_000 });
  await sleep(HOLD);
  await go("home-personal", "page-org");
  await page.getByTestId("crumb-home").click();
  await page.getByTestId("page-home").waitFor();
  await sleep(HOLD);
  await go("home-org", "page-org", HOLD_LATER);
  await go("org-boxes", "page-boxes", HOLD_LATER);
  await go("card-box-1", "page-box", HOLD_LATER);
  await go("nav-people", "page-people", HOLD_LATER);
  await go("card-alice", "page-person", HOLD_LATER);
  await go("nav-access", "page-access", HOLD_LATER);
  await go("grant-access", "page-grant", HOLD_LATER);
  await go("nav-home", "page-home", HOLD);
  await click("theme");
  await sleep(HOLD_LATER);
  await go("home-org", "page-org", HOLD_LATER);
  await go("org-boxes", "page-boxes", HOLD_LATER);
  await go("nav-people", "page-people", HOLD_LATER);
  await go("nav-access", "page-access", HOLD_LATER);
  await click("sign-out");
  await page.getByTestId("gate").waitFor();
  await sleep(HOLD_LATER);
} catch (err) {
  await page.screenshot({ path: path.join(DIR, "fail.png") });
  fs.writeFileSync(path.join(DIR, "fail.html"), await page.content());
  throw err;
} finally {
  const video = page.video();
  await context.close();
  await browser.close();
  if (video) {
    const src = await video.path();
    const dest = path.join(DIR, "raw.webm");
    if (src !== dest) fs.renameSync(src, dest);
    console.log(dest);
  }
}
