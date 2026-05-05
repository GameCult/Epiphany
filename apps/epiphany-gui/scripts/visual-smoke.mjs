import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { createServer } from "vite";

const root = resolve(import.meta.dirname, "..", "..", "..");
const artifactDir = resolve(root, ".epiphany-gui");
const desktopScreenshotPath = resolve(artifactDir, "operator-console-smoke-desktop.png");
const wideScreenshotPath = resolve(artifactDir, "operator-console-smoke-wide.png");
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
  const browser = await launchSmokeBrowser();
  const wide = await smokeViewport(browser, { width: 2048, height: 1024 }, wideScreenshotPath, false);
  const desktop = await smokeViewport(browser, { width: 1366, height: 900 }, desktopScreenshotPath, true);
  const mobile = await smokeViewport(browser, { width: 390, height: 844 }, mobileScreenshotPath, false);
  await browser.close();
  console.log(
    JSON.stringify(
      {
        ok: true,
        screenshots: [wideScreenshotPath, desktopScreenshotPath, mobileScreenshotPath],
        probes: { wide, desktop, mobile },
      },
      null,
      2,
    ),
  );
} finally {
  await server.close();
}

async function launchSmokeBrowser() {
  const requestedChannel = process.env.EPIPHANY_SMOKE_BROWSER;
  const channels = requestedChannel ? [requestedChannel] : ["chrome", "msedge"];
  let lastError = null;
  for (const channel of channels) {
    try {
      return await chromium.launch({ channel, headless: true });
    } catch (error) {
      lastError = error;
    }
  }
  throw lastError;
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

  const operatorSurface = await page.evaluate(() => {
    return [".immersiveTopbar", ".deckRail", ".diegeticPanel"].every((selector) => {
      const element = document.querySelector(selector);
      if (!element) return false;
      const style = window.getComputedStyle(element);
      const rect = element.getBoundingClientRect();
      return (
        style.opacity !== "0" &&
        style.visibility !== "hidden" &&
        rect.width > 20 &&
        rect.height > 20
      );
    });
  });
  if (!operatorSurface) {
    throw new Error("static operator overlays are not visible");
  }
  if (!crispProbe.nonBlank && !operatorSurface) {
    throw new Error(`no crisp operator surface rendered: ${crispProbe.reason}`);
  }

  const projectionProbe = await page.evaluate(() => {
    const node = document.querySelector('[data-agent-node="coordinator"]');
    if (!(node instanceof HTMLElement)) return { ok: false, reason: "coordinator DOM node missing" };
    const style = node.style;
    const x = style.getPropertyValue("--agent-x");
    const y = style.getPropertyValue("--agent-y");
    const glow = Number.parseFloat(style.getPropertyValue("--agent-glow-pulse"));
    return {
      ok: x.endsWith("%") && y.endsWith("%") && Number.isFinite(glow),
      reason: `x=${x} y=${y} glow=${glow}`,
    };
  });
  if (!projectionProbe.ok) {
    throw new Error(`DOM agent projection was not synchronized: ${projectionProbe.reason}`);
  }

  const hoverBox = await page.locator('[data-agent-node="research"]').boundingBox();
  if (!hoverBox) {
    throw new Error("research DOM node has no hover bounds");
  }
  await page.mouse.move(hoverBox.x + hoverBox.width / 2, hoverBox.y + hoverBox.height / 2);
  await page.waitForTimeout(180);
  const hoverProbe = await page.evaluate(() => {
    const node = document.querySelector('[data-agent-node="research"]');
    if (!(node instanceof HTMLElement)) return { ok: false, reason: "research DOM node missing" };
    const hover = Number.parseFloat(node.style.getPropertyValue("--agent-hover"));
    const acknowledgement = Number.parseFloat(node.style.getPropertyValue("--agent-ack"));
    return {
      ok: hover > 0.7 && acknowledgement >= 0,
      reason: `hover=${hover} ack=${acknowledgement}`,
    };
  });
  if (!hoverProbe.ok) {
    throw new Error(`DOM hover did not reach aquarium projection: ${hoverProbe.reason}`);
  }
  const researchBox = await page.locator('[data-agent-node="research"]').boundingBox();
  if (!researchBox) {
    throw new Error("research DOM node has no clickable bounds");
  }
  await page.mouse.click(researchBox.x + researchBox.width / 2, researchBox.y + researchBox.height / 2);
  try {
    await page.waitForFunction(() => {
      const audio = window.__epiphanyAquariumAudio;
      return audio?.state === "running" &&
        audio.vocalAgentCount >= 7 &&
        audio.lastBurstChirpDrivers >= 6 &&
        audio.spectral?.chirpDrivers === 6 &&
        audio.spectral?.lastBurstChoirVoices >= 3 &&
        audio.spectral?.reactiveFlushes >= 1 &&
        audio.spectral?.transientBins >= 24 &&
        audio.spectral?.vocalAgents >= 7 &&
        audio.spectral?.queuedFrames >= 2048 &&
        audio.lastBurst;
    }, null, { timeout: 5000 });
  } catch (error) {
    const audio = await page.evaluate(() => window.__epiphanyAquariumAudio ?? null);
    throw new Error(`aquarium audio did not wake correctly: ${JSON.stringify(audio)}`, { cause: error });
  }
  const audioProbe = await page.evaluate(() => window.__epiphanyAquariumAudio ?? null);

  let persistedParams = null;
  if (exerciseFluidPanel) {
    await page.evaluate((key) => window.localStorage.removeItem(key), fluidStorageKey);
    await page.reload({ waitUntil: "networkidle" });
    await page.locator(".agentSmokeCanvas").waitFor();
    await page.waitForTimeout(700);
    const inspectorGuard = viewport.width >= 720 ? Math.min(230, viewport.height * 0.25) : 0;
    await page.mouse.click(viewport.width - 48, viewport.height - 48 - inspectorGuard);
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
  return { smokeProbe, crispProbe, audioProbe, persistedParams };
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
    const pixels = context.getImageData(0, 0, canvas.width, canvas.height).data;
    const stride = Math.max(4, Math.floor(pixels.length / 4096 / 4) * 4);
    let nonBlank = false;
    for (let index = 0; index < pixels.length - 3; index += stride) {
      if (pixels[index] !== 0 || pixels[index + 1] !== 0 || pixels[index + 2] !== 0 || pixels[index + 3] !== 0) {
        nonBlank = true;
        break;
      }
    }
    return {
      nonBlank,
      reason: "2d whole-canvas sample",
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
