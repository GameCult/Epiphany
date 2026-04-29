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
  await page.getByText("prepareCheckpoint").waitFor();
  await page.getByText("Role Lanes").waitFor();
  await page.getByText("Artifact Bundles").waitFor();
  if (viewport.width >= 900) {
    await page.getByRole("button", { name: "Status Snapshot" }).click();
    await page.getByText("statusSnapshot sample completed.").waitFor();
    await page.getByRole("button", { name: "Coordinator Plan" }).click();
    await page.getByText("coordinatorPlan sample completed.").waitFor();
  }
  await page.screenshot({ path: screenshotPath, fullPage: true });

  const result = await page.evaluate(() => {
    const elements = Array.from(document.querySelectorAll("h1, h2, h3, p, dd, code, button, input, .pill"));
    const overlaps = [];
    for (let index = 0; index < elements.length; index += 1) {
      const left = elements[index];
      const a = elements[index].getBoundingClientRect();
      if (a.width === 0 || a.height === 0) continue;
      for (let other = index + 1; other < elements.length; other += 1) {
        const right = elements[other];
        if (left.contains(right) || right.contains(left)) continue;
        const b = elements[other].getBoundingClientRect();
        if (b.width === 0 || b.height === 0) continue;
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
