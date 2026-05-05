import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { createServer } from "vite";

const root = resolve(import.meta.dirname, "..", "..", "..");
const artifactDir = resolve(root, ".epiphany-gui");
const desktopScreenshotPath = resolve(artifactDir, "operator-console-smoke-desktop.png");
const mobileScreenshotPath = resolve(artifactDir, "operator-console-smoke-mobile.png");
const url = "http://127.0.0.1:1420";
const fluidStorageKey = "epiphany:aquarium-fluid-params:v3";

await mkdir(artifactDir, { recursive: true });

const server = await createServer({
  root: resolve(root, "apps", "epiphany-gui"),
  logLevel: "silent",
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
  },
});

try {
  await server.listen();
  await waitForServer(url);
  const browser = await chromium.launch({
    channel: "msedge",
    headless: true,
  });
  const desktop = await smokeViewport(browser, { width: 1366, height: 900 }, desktopScreenshotPath, true);
  const mobile = await smokeViewport(browser, { width: 390, height: 844 }, mobileScreenshotPath, false);
  await browser.close();
  console.log(
    JSON.stringify(
      {
        ok: true,
        screenshots: [desktopScreenshotPath, mobileScreenshotPath],
        probes: { desktop, mobile },
      },
      null,
      2,
    ),
  );
} finally {
  await server.close();
}

async function smokeViewport(browser, viewport, screenshotPath, exerciseFluidPanel) {
  const page = await browser.newPage({ viewport });
  await page.goto(url, { waitUntil: "networkidle" });
  await page.locator("h1").filter({ hasText: "Operator Console" }).waitFor({ state: "attached" });
  await page.locator(".immersiveShell").waitFor();
  await page.locator(".agentSmokeCanvas").waitFor();
  await page.locator(".agentCrispCanvas").waitFor();
  await page.waitForTimeout(700);

  const smokeProbe = await probeCanvas(page, ".agentSmokeCanvas");
  const crispProbe = await probeCanvas(page, ".agentCrispCanvas");
  if (!smokeProbe.nonBlank) {
    throw new Error(`agent smoke canvas did not render: ${smokeProbe.reason}`);
  }
  if (!crispProbe.nonBlank) {
    throw new Error(`agent crisp canvas did not render: ${crispProbe.reason}`);
  }

  const hiddenSurface = await page.evaluate(() => {
    return [".immersiveTopbar", ".deckRail", ".diegeticPanel", ".hudToastStack"].every((selector) => {
      const element = document.querySelector(selector);
      if (!element) return false;
      const style = window.getComputedStyle(element);
      return style.opacity === "0" && style.pointerEvents === "none";
    });
  });
  if (!hiddenSurface) {
    throw new Error("static operator overlays are still visible or interactive");
  }

  let persistedParams = null;
  if (exerciseFluidPanel) {
    await page.evaluate((key) => window.localStorage.removeItem(key), fluidStorageKey);
    await page.reload({ waitUntil: "networkidle" });
    await page.locator(".agentSmokeCanvas").waitFor();
    await page.waitForTimeout(700);
    await page.mouse.click(viewport.width - 48, viewport.height - 48);
    await page.waitForTimeout(120);
    const railY = Math.round(viewport.height * 0.47);
    await page.mouse.move(viewport.width - 226, railY);
    await page.mouse.down();
    await page.mouse.move(viewport.width - 46, railY, { steps: 8 });
    await page.mouse.up();
    persistedParams = await page.evaluate((key) => window.localStorage.getItem(key), fluidStorageKey);
    if (!persistedParams || !persistedParams.includes("timeScale")) {
      throw new Error("fluid parameter panel did not persist changed parameters");
    }
  }

  await page.screenshot({ path: screenshotPath, fullPage: true });
  const result = await page.evaluate(() => ({
    horizontalOverflow: document.documentElement.scrollWidth > document.documentElement.clientWidth + 1,
  }));
  await page.close();
  if (result.horizontalOverflow) {
    throw new Error(`visual smoke found horizontal overflow at ${viewport.width}x${viewport.height}`);
  }
  return { smokeProbe, crispProbe, persistedParams };
}

async function probeCanvas(page, selector) {
  return page.locator(selector).evaluate((canvas) => {
    if (!(canvas instanceof HTMLCanvasElement) || canvas.width === 0 || canvas.height === 0) {
      return { nonBlank: false, reason: "canvas has no drawable dimensions" };
    }
    const gl = canvas.getContext("webgl2");
    if (gl) {
      const width = Math.min(12, canvas.width);
      const height = Math.min(12, canvas.height);
      const pixels = new Uint8Array(width * height * 4);
      gl.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
      return {
        nonBlank: pixels.some((value) => value !== 0),
        reason: "webgl2 sample",
      };
    }
    const context = canvas.getContext("2d");
    if (!context) return { nonBlank: false, reason: "no readable canvas context" };
    const width = Math.min(64, canvas.width);
    const height = Math.min(64, canvas.height);
    const centerX = Math.max(0, Math.floor(canvas.width / 2 - width / 2));
    const centerY = Math.max(0, Math.floor(canvas.height / 2 - height / 2));
    const pixels = context.getImageData(centerX, centerY, width, height).data;
    return {
      nonBlank: Array.from(pixels).some((value) => value !== 0),
      reason: "2d sample",
    };
  });
}

async function waitForServer(target) {
  const deadline = Date.now() + 30000;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(target);
      if (response.ok) return;
    } catch {
      // Server is still waking up.
    }
    await new Promise((resolveWait) => setTimeout(resolveWait, 500));
  }
  throw new Error("Vite server did not start.");
}
