import { chromium } from "playwright";
const URL = "http://127.0.0.1:5173/";
const browser = await chromium.launch({ headless: false, args: ["--ignore-gpu-blocklist", "--enable-gpu", "--no-sandbox"] });
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
const errs = [];
page.on("console", (m) => m.type() === "error" && errs.push(m.text()));
page.on("pageerror", (e) => errs.push(`pageerror ${e.message}`));
await page.goto(URL, { waitUntil: "networkidle" });
await page.waitForTimeout(3500);

const state = () =>
  page.evaluate(() => {
    const t = document.body.innerText;
    return {
      energy: (t.match(/System energy\s*[^\n]+/) || [""])[0].replace(/\s+/g, " ").trim(),
      nodes: (t.match(/MN\d/g) || []).length,
      totalDv: (t.match(/TOTAL ΔV\s*[\d.]+ m\/s/) || [""])[0].replace(/\s+/g, " ").trim(),
    };
  });

console.log("load (paused):", JSON.stringify(await state()));

// Orbit click: sweep points; report node count growth.
const box = await page.locator("canvas").boundingBox();
const pts = [];
for (let i = 0; i < 24; i++) pts.push([0.12 + 0.03 * i, 0.5 + 0.18 * Math.sin(i * 0.6)]);
let placed = 0;
for (const [fx, fy] of pts) {
  const before = (await state()).nodes;
  await page.mouse.click(box.x + box.width * fx, box.y + box.height * fy);
  await page.waitForTimeout(250);
  const after = (await state()).nodes;
  if (after > before) {
    placed++;
    console.log(`  orbit click hit at (${fx.toFixed(2)},${fy.toFixed(2)}) -> nodes ${after}`);
    break;
  }
}
console.log("orbit-click placed a node:", placed > 0);
await page.screenshot({ path: "/tmp/verify-orbitclick.png" });

// Clear, then Hohmann pair selects departure (panel shows node Δv, not draft 500).
await page.evaluate(() => document.activeElement?.blur?.());
await page.keyboard.press("KeyC");
await page.waitForTimeout(400);
await page.keyboard.down("Shift");
await page.keyboard.press("KeyH");
await page.keyboard.up("Shift");
await page.waitForTimeout(1200);
console.log("after Shift+H:", JSON.stringify(await state()));
await page.screenshot({ path: "/tmp/verify-hohmann.png" });

console.log("errors:", errs.length ? errs.join("\n") : "(none)");
await browser.close();
