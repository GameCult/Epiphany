import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { createServer } from "vite";

const root = resolve(import.meta.dirname, "..", "..", "..");
const artifactDir = resolve(root, ".epiphany-gui");
const desktopScreenshotPath = resolve(artifactDir, "operator-console-smoke-desktop.png");
const mobileScreenshotPath = resolve(artifactDir, "operator-console-smoke-mobile.png");
const url = "http://127.0.0.1:1420";

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
  await smokeViewport(browser, { width: 1366, height: 900 }, desktopScreenshotPath);
  await smokeViewport(browser, { width: 390, height: 844 }, mobileScreenshotPath);
  await browser.close();
  console.log(
    JSON.stringify(
      {
        ok: true,
        screenshots: [desktopScreenshotPath, mobileScreenshotPath],
      },
      null,
      2,
    ),
  );
} finally {
  await server.close();
}

async function smokeViewport(browser, viewport, screenshotPath) {
  const page = await browser.newPage({ viewport });
  await page.goto(url, { waitUntil: "networkidle" });
  await page.getByRole("heading", { name: "Operator Console" }).waitFor();
  await page.locator(".immersiveShell").waitFor();
  await page.getByRole("button", { name: "Self prepareCheckpoint" }).waitFor();
  await page.locator(".agentSmokeCanvas").waitFor();
  await page.waitForTimeout(350);
  const canvasProbe = await page.locator(".agentSmokeCanvas").evaluate((canvas) => {
    if (!(canvas instanceof HTMLCanvasElement) || canvas.width === 0 || canvas.height === 0) {
      return { nonBlank: false, reason: "canvas has no drawable dimensions" };
    }
    const gl = canvas.getContext("webgl2");
    if (gl) {
      const width = Math.min(6, canvas.width);
      const height = Math.min(6, canvas.height);
      const pixels = new Uint8Array(width * height * 4);
      gl.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
      return {
        nonBlank: pixels.some((value) => value !== 0),
        reason: "webgl2 sample",
      };
    }
    const context = canvas.getContext("2d");
    if (!context) return { nonBlank: false, reason: "no readable canvas context" };
    const width = Math.min(6, canvas.width);
    const height = Math.min(6, canvas.height);
    const pixels = context.getImageData(0, 0, width, height).data;
    return {
      nonBlank: Array.from(pixels).some((value) => value !== 0),
      reason: "2d sample",
    };
  });
  if (!canvasProbe.nonBlank) {
    throw new Error(`agent smoke canvas did not render: ${canvasProbe.reason}`);
  }
  await page.getByRole("button", { name: "Command" }).waitFor();
  await page.getByRole("button", { name: "State" }).waitFor();
  await page.getByRole("button", { name: "Agents" }).waitFor();
  await page.getByRole("button", { name: "Artifacts" }).waitFor();
  await page.getByRole("button", { name: "Prepare Checkpoint" }).waitFor();
  await page.getByRole("button", { name: "Inspect Rider" }).waitFor();
  await page.getByRole("button", { name: "Adopt Draft" }).waitFor();
  await page.getByRole("button", { name: "Launch Imagination" }).waitFor();
  await page.getByRole("button", { name: "Read Imagination" }).waitFor();
  await page.getByRole("button", { name: "Launch Modeling" }).waitFor();
  await page.getByRole("button", { name: "Read Modeling" }).waitFor();
  await page.getByRole("button", { name: "Launch Verification" }).waitFor();
  await page.getByRole("button", { name: "Read Verification" }).waitFor();
  await page.getByRole("button", { name: "Launch Reorient" }).waitFor();
  await page.getByRole("button", { name: "Read Reorient" }).waitFor();
  await page.getByRole("button", { name: "Accept Reorient" }).waitFor();
  await page.getByRole("button", { name: "State" }).click();
  await page.getByRole("button", { name: "environment" }).waitFor();
  await page.getByText("Unity Editor").waitFor();
  await page.getByRole("heading", { name: "Rider", exact: true }).waitFor();
  await page.getByText("Aetheria.sln").waitFor();
  await page.getByRole("button", { name: "planning" }).click();
  await page.getByRole("heading", { name: "Build the planning dashboard slice", exact: true }).waitFor();
  await page.getByRole("button", { name: "graph" }).click();
  await page.getByText("Epiphany Typed Graph").waitFor();
  await page.getByRole("button", { name: "Agents" }).click();
  await page.getByRole("button", { name: "lanes" }).waitFor();
  await page.getByRole("heading", { name: "Implementation", exact: true }).waitFor();
  await page.getByRole("button", { name: "findings" }).click();
  await page.getByRole("heading", { name: "Imagination / Planning", exact: true }).waitFor();
  await page.getByRole("button", { name: "jobs" }).click();
  await page.getByText("retrieval-index").waitFor();
  await page.getByRole("button", { name: "Artifacts" }).click();
  await page.getByText("runtime/unity-inspect-sample").waitFor();
  await page.getByRole("button", { name: "Command" }).click();
  if (viewport.width >= 900) {
    await page.getByRole("button", { name: "Status Snapshot" }).click();
    await page.getByText("statusSnapshot sample completed.").waitFor();
    await page.getByRole("button", { name: "Coordinator Plan" }).click();
    await page.getByText("coordinatorPlan sample completed.").waitFor();
    await page.getByRole("button", { name: "Inspect Rider" }).click();
    await page.getByText("inspectRider sample completed.").waitFor();
    await page.getByRole("button", { name: "Prepare Checkpoint" }).click();
    await page.getByText("prepareCheckpoint sample completed.").waitFor();
    await page.getByRole("button", { name: "Read Imagination" }).click();
    await page.getByText("readImaginationResult sample completed.").waitFor();
    await page.getByRole("button", { name: "Read Modeling" }).click();
    await page.getByText("readModelingResult sample completed.").waitFor();
    await page.getByRole("button", { name: "Read Reorient" }).click();
    await page.getByText("readReorientResult sample completed.").waitFor();
  }
  await page.screenshot({ path: screenshotPath, fullPage: true });

  const result = await page.evaluate(() => {
    const elements = Array.from(document.querySelectorAll("h1, h2, h3, p, dd, code, button, input, .pill"));
    const overlaps = [];
    function isPaintedAtSample(element, rect) {
      const style = window.getComputedStyle(element);
      if (style.visibility === "hidden" || style.display === "none" || Number(style.opacity) === 0) {
        return false;
      }
      const points = [
        [rect.left + rect.width / 2, rect.top + rect.height / 2],
        [rect.left + Math.min(8, rect.width / 2), rect.top + rect.height / 2],
        [rect.right - Math.min(8, rect.width / 2), rect.top + rect.height / 2],
        [rect.left + rect.width / 2, rect.top + Math.min(8, rect.height / 2)],
        [rect.left + rect.width / 2, rect.bottom - Math.min(8, rect.height / 2)],
      ];
      return points.some(([x, y]) => {
        if (x < 0 || y < 0 || x > window.innerWidth || y > window.innerHeight) return false;
        const hit = document.elementFromPoint(x, y);
        return hit === element || Boolean(hit && element.contains(hit));
      });
    }
    for (let index = 0; index < elements.length; index += 1) {
      const left = elements[index];
      const a = elements[index].getBoundingClientRect();
      if (a.width === 0 || a.height === 0) continue;
      if (!isPaintedAtSample(left, a)) continue;
      for (let other = index + 1; other < elements.length; other += 1) {
        const right = elements[other];
        if (left.contains(right) || right.contains(left)) continue;
        const b = elements[other].getBoundingClientRect();
        if (b.width === 0 || b.height === 0) continue;
        if (!isPaintedAtSample(right, b)) continue;
        const x = Math.min(a.right, b.right) - Math.max(a.left, b.left);
        const y = Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top);
        if (x > 2 && y > 2) {
          overlaps.push({
            left: left.textContent?.trim() || left.getAttribute("placeholder") || left.tagName,
            right: right.textContent?.trim() || right.getAttribute("placeholder") || right.tagName,
          });
        }
      }
    }
    return {
      overlaps,
      horizontalOverflow: document.documentElement.scrollWidth > document.documentElement.clientWidth + 1,
    };
  });

  await page.close();
  if (result.horizontalOverflow) {
    throw new Error(`visual smoke found horizontal overflow at ${viewport.width}x${viewport.height}`);
  }
  if (result.overlaps.length > 0) {
    throw new Error(`visual smoke found overlapping text/control boxes: ${JSON.stringify(result.overlaps)}`);
  }
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
