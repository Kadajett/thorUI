import { expect, test } from "@playwright/test";

const profiles = [
  { name: "main", width: 1920, height: 1080 },
  { name: "companion", width: 1240, height: 1080 },
];
const isDeployed = Boolean(process.env.THORUI_BASE_URL);

for (const profile of profiles) {
  test(`${profile.name} surface boots and exports observations`, async ({ page }) => {
    await page.setViewportSize(profile);
    await page.goto(`/?surface=${profile.name}&session=browser-test`);
    await expect(page.locator("h1")).toHaveText("Capability Lab");
    await expect(page.locator("#surface-role")).toContainText(profile.name);
    await expect(page.locator("#metric-viewport")).toContainText(`${profile.width} × ${profile.height}`);
    const report = await reportJson(page);
    expect(report.schema_version).toBe(1);
    expect(report.surface.role).toBe(profile.name);
    expect(report.surface.viewport_width_css).toBe(profile.width);
    expect(report.support.webgl2).toBe(true);
    await expect.poll(async () => (await reportJson(page)).active_probes.service_worker_result).toBe("registered");
  });
}

test("two surfaces discover each other and preserve separate roles", async ({ page, context }) => {
  await page.goto("/?surface=main&session=peer-test");
  const companion = await context.newPage();
  await companion.goto("/?surface=companion&session=peer-test");
  await expect(companion.locator("#surface-role")).toContainText("companion");
  await expect(page.locator("#connection-badge")).toHaveText("Peer linked");
  await page.locator("#ping-peer").click();
  await expect(page.locator("#peer-summary")).toContainText("received", { timeout: 10_000 });
});

test("pointer capture and frame sampling enter the report", async ({ page }) => {
  await page.goto("/?surface=main&session=input-test");
  await reportJson(page);
  await page.locator("#touch-target").evaluate((target) => {
    target.dispatchEvent(new PointerEvent("pointerdown", {
      bubbles: true,
      clientX: 40,
      clientY: 50,
      isPrimary: true,
      pointerId: 7,
      pointerType: "touch",
      pressure: 0.7,
    }));
  });
  await expect(page.locator("#pointer-summary")).toContainText("touch #7");
  await page.locator(".manual-probes summary").click();
  await page.locator("#measure-frames").click();
  await expect(page.locator("#metric-frame")).toContainText("Hz", { timeout: 10_000 });
  const report = await reportJson(page);
  expect(report.pointer_samples).toHaveLength(1);
  expect(report.frame_runs).toHaveLength(1);
  expect(report.frame_runs[0].distribution.samples).toBeGreaterThan(10);
});

test("controller navigation moves focus and activates with A", async ({ page }) => {
  await installGamepad(page);
  await page.goto("/?surface=main&session=controller-test");
  await setGamepadButton(page, 13, true);
  await expect(page.locator("#run-suite")).toBeFocused();
  await expect(page.locator("#controller-nav-status")).toContainText("Test Controller");
  await setGamepadButton(page, 13, false);
  await page.evaluate(() => {
    window.__thorActivated = false;
    document.querySelector("#run-suite").addEventListener("click", (event) => {
      window.__thorActivated = true;
      event.stopImmediatePropagation();
    }, { capture: true });
  });
  await setGamepadButton(page, 0, true);
  await expect.poll(() => page.evaluate(() => window.__thorActivated)).toBe(true);
});

test("one action runs the guided suite and saves a receipt", async ({ page }) => {
  test.skip(isDeployed, "The deployed API has a separate fast smoke test");
  await page.goto("/?surface=main&session=suite-test");
  await reportJson(page);
  await page.locator("#run-suite").click();
  await page.locator("#touch-target").evaluate((target) => {
    target.dispatchEvent(new PointerEvent("pointerdown", {
      bubbles: true,
      isPrimary: true,
      pointerId: 9,
      pointerType: "touch",
    }));
  });
  await expect(page.locator("#suite-status")).toContainText("Saved. Receipt", { timeout: 28_000 });
});

test("deployed report API accepts a same-origin capability report", async ({ request }) => {
  const baseUrl = process.env.THORUI_BASE_URL;
  test.skip(!baseUrl, "The local static server does not run the Worker API");
  const response = await request.post(`${baseUrl}/api/reports`, {
    headers: { Origin: baseUrl },
    data: {
      schema_version: 1,
      capture_id: "automated-smoke-report",
      captured_at: new Date().toISOString(),
      build: { revision: "browser-smoke" },
      surface: { role: "main", session_id: "browser-smoke-session" },
    },
  });
  expect(response.status()).toBe(201);
  expect((await response.json()).receipt_id).toMatch(/^[a-f0-9]{8}$/);
});

async function reportJson(page) {
  await expect(page.locator("#report-json")).not.toHaveText("{}");
  return JSON.parse(await page.locator("#report-json").textContent());
}

async function installGamepad(page) {
  await page.addInitScript(() => {
    const buttons = Array.from({ length: 17 }, () => ({ pressed: false, touched: false, value: 0 }));
    window.__thorGamepad = {
      axes: [0, 0, 0, 0],
      buttons,
      connected: true,
      id: "Test Controller",
      index: 0,
      mapping: "standard",
      timestamp: 0,
    };
    Object.defineProperty(navigator, "getGamepads", {
      configurable: true,
      value: () => [window.__thorGamepad],
    });
  });
}

async function setGamepadButton(page, index, pressed) {
  await page.evaluate(({ buttonIndex, value }) => {
    const button = window.__thorGamepad.buttons[buttonIndex];
    button.pressed = value;
    button.touched = value;
    button.value = value ? 1 : 0;
    window.__thorGamepad.timestamp += 1;
  }, { buttonIndex: index, value: pressed });
  await page.waitForTimeout(80);
}
