import { defineConfig } from "@playwright/test";

const externalBaseUrl = process.env.THORUI_BASE_URL;
const baseURL = externalBaseUrl ?? "http://127.0.0.1:8787";

export default defineConfig({
  testDir: "tests/browser",
  fullyParallel: false,
  timeout: 30_000,
  reporter: "line",
  use: {
    baseURL,
    browserName: "chromium",
    headless: true,
    launchOptions: { executablePath: "/usr/bin/google-chrome" },
  },
  webServer: externalBaseUrl
    ? undefined
    : {
        command: "pnpm exec wrangler dev --local --port 8787",
        url: baseURL,
        reuseExistingServer: false,
      },
});
