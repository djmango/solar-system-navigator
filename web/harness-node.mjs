// Interactive validation: create a maneuver node and inspect the gizmo.
import { chromium } from "playwright";

const URL = process.env.HARNESS_URL ?? "http://127.0.0.1:5173/";
const browser = await chromium.launch({
  headless: false,
  args: ["--ignore-gpu-blocklist", "--enable-gpu", "--no-sandbox"],
});
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
const logs = [];
page.on("console", (m) => logs.push(`[${m.type()}] ${m.text()}`));
page.on("pageerror", (e) => logs.push(`[pageerror] ${e.message}`));

await page.goto(URL, { waitUntil: "networkidle", timeout: 60000 });
await page.waitForTimeout(3500);

// Create a maneuver node (auto-selected) → gizmo should appear on the node.
await page.click('button:has-text("Create node")');
await page.waitForTimeout(1500);
await page.screenshot({ path: "/tmp/node-created.png" });

// Bump prograde via the planner input to reshape the amber path.
const progradeInput = page.locator('input[type="number"]').first();
await progradeInput.fill("1500");
await progradeInput.press("Enter");
await page.waitForTimeout(1500);
await page.screenshot({ path: "/tmp/node-prograde.png" });

// Zoom out to confirm the gizmo keeps a constant screen size.
const box = await page.locator("canvas").boundingBox();
await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
for (let i = 0; i < 6; i++) {
  await page.mouse.wheel(0, 400);
  await page.waitForTimeout(120);
}
await page.waitForTimeout(800);
await page.screenshot({ path: "/tmp/node-zoomout.png" });

const planner = await page.evaluate(() => {
  // @ts-ignore
  const s = window.__SIM_STORE__?.getState?.();
  return s ? { nodes: s.planner?.nodes?.length, sel: s.selectedNodeIndex } : "no store";
});
console.log("planner:", JSON.stringify(planner));
console.log("=== console (last 20) ===");
console.log(logs.slice(-20).join("\n"));
await browser.close();
