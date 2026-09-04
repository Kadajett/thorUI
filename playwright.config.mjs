import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "tests/browser",
  fullyParallel: false,
  timeout: 30_000,
  reporter: "line",
  use: {
    baseURL: "http://127.0.0.1:8787",
    browserName: "chromium",
    headless: true,
    launchOptions: { executablePath: "/usr/bin/google-chrome" },
  },
  webServer: {
    command: "pnpm exec wrangler dev --local --port 8787",
    url: "http://127.0.0.1:8787",
    reuseExistingServer: false,
  },
});

