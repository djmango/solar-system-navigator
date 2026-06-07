// Validate node-time sliding + SOI-aware framing.
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
await page.screenshot({ path: "/tmp/slide-0-framed.png" });

await page.click('button:has-text("Create node")');
await page.waitForTimeout(1200);
await page.screenshot({ path: "/tmp/slide-1-node.png" });

// Slide the node far along its orbit via the range slider (keyboard End → commit).
const slider = page.locator('input[type="range"]');
if (await slider.count()) {
  await slider.first().focus();
  await page.keyboard.press("End");
  await page.waitForTimeout(200);
  await page.keyboard.up("End");
  await page.waitForTimeout(1500);
  await page.screenshot({ path: "/tmp/slide-2-far.png" });
  // Bring it back to ~1/3 along.
  for (let i = 0; i < 14; i++) await page.keyboard.press("ArrowLeft");
  await page.keyboard.up("ArrowLeft");
  await page.waitForTimeout(1500);
  await page.screenshot({ path: "/tmp/slide-3-mid.png" });
}

const info = await page.evaluate(() => document.body.innerText.match(/Node UT[\s\S]{0,40}/)?.[0] ?? "");
console.log("nodeUT:", JSON.stringify(info));
console.log(logs.filter((l) => l.includes("error") || l.includes("pageerror")).slice(-10).join("\n") || "(no errors)");
await browser.close();
