// Headless validation harness for the web app.
// Usage: node harness.mjs [outPrefix]
import { chromium } from "playwright";

const URL = process.env.HARNESS_URL ?? "http://127.0.0.1:5173/";
const OUT = process.argv[2] ?? "/tmp/web";

const headed = process.env.HARNESS_HEADED === "1";
const browser = await chromium.launch({
  headless: !headed,
  args: headed
    ? ["--ignore-gpu-blocklist", "--enable-gpu", "--no-sandbox"]
    : [
        "--use-gl=angle",
        "--use-angle=swiftshader",
        "--enable-unsafe-swiftshader",
        "--ignore-gpu-blocklist",
      ],
});
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });

const logs = [];
const failed = [];
page.on("console", (m) => logs.push(`[${m.type()}] ${m.text()}`));
page.on("pageerror", (e) => logs.push(`[pageerror] ${e.message}`));
page.on("requestfailed", (r) =>
  failed.push(`${r.failure()?.errorText ?? "?"} ${r.url()}`),
);
page.on("response", (r) => {
  if (r.status() >= 400) failed.push(`HTTP ${r.status()} ${r.url()}`);
});

await page.goto(URL, { waitUntil: "networkidle", timeout: 60000 });
await page.waitForTimeout(4000);
await page.screenshot({ path: `${OUT}-load.png` });

// Report WebGL availability + canvas presence.
const info = await page.evaluate(() => {
  const c = document.querySelector("canvas");
  let gl = null;
  try {
    const t = document.createElement("canvas");
    gl = t.getContext("webgl2") || t.getContext("webgl");
  } catch {}
  return {
    hasCanvas: !!c,
    canvasSize: c ? [c.width, c.height] : null,
    webgl: !!gl,
    renderer: gl ? gl.getParameter(gl.RENDERER) : null,
    bodyText: document.body.innerText.slice(0, 600),
  };
});

console.log("=== PAGE INFO ===");
console.log(JSON.stringify(info, null, 2));
console.log("=== FAILED REQUESTS ===");
console.log(failed.length ? failed.join("\n") : "(none)");
console.log("=== CONSOLE (last 40) ===");
console.log(logs.slice(-40).join("\n"));

await browser.close();
