import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: ".",
  timeout: 60_000,
  expect: { timeout: 15_000 },
  fullyParallel: false,
  workers: 1,
  retries: 0,
  testMatch: ["console.spec.ts"],
  use: {
    baseURL: process.env.CONSOLE_URL ?? "http://127.0.0.1:3040",
    headless: true,
    viewport: { width: 1280, height: 800 },
  },
});
