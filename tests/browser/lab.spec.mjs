import { expect, test } from "@playwright/test";

const profiles = [
  { name: "main", width: 1920, height: 1080 },
  { name: "companion", width: 1240, height: 1080 },
];

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

test("two surfaces discover each other and preserve separate roles", async ({ page }) => {
  await page.goto("/?surface=main&session=peer-test");
  const popupPromise = page.waitForEvent("popup");
  await page.locator("#open-companion").click();
  const companion = await popupPromise;
  await expect(companion.locator("#surface-role")).toContainText("companion");
  await expect(page.locator("#connection-badge")).toHaveText("Peer linked");
  await page.locator("#ping-peer").click();
  await expect(page.locator("#peer-summary")).toContainText("received", { timeout: 10_000 });
});

test("pointer capture and frame sampling enter the report", async ({ page }) => {
  await page.goto("/?surface=main&session=input-test");
  await page.locator("#touch-target").dispatchEvent("pointerdown", {
    pointerId: 7,
    pointerType: "touch",
    clientX: 40,
    clientY: 50,
    pressure: 0.7,
  });
  await expect(page.locator("#pointer-summary")).toContainText("touch #7");
  await page.locator("#measure-frames").click();
  await expect(page.locator("#metric-frame")).toContainText("Hz", { timeout: 10_000 });
  const report = await reportJson(page);
  expect(report.pointer_samples).toHaveLength(1);
  expect(report.frame_runs).toHaveLength(1);
  expect(report.frame_runs[0].distribution.samples).toBeGreaterThan(10);
});

async function reportJson(page) {
  await expect(page.locator("#report-json")).not.toHaveText("{}");
  return JSON.parse(await page.locator("#report-json").textContent());
}
