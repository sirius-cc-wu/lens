import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "tests/browser",
  globalSetup: "./tests/browser/global-setup.mjs",
  fullyParallel: false,
  workers: 1,
  use: {
    browserName: "chromium",
    channel: "chrome",
    headless: true,
  },
});
