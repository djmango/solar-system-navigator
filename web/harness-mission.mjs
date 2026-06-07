// Full-mission audit: drive the planner like a player and record what works.
import { chromium } from "playwright";

const URL = process.env.HARNESS_URL ?? "http://127.0.0.1:5173/";
const OUT = "/tmp/mission";
const browser = await chromium.launch({
  headless: false,
  args: ["--ignore-gpu-blocklist", "--enable-gpu", "--no-sandbox"],
});
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
const errors = [];
const net = [];
page.on("console", (m) => {
  if (m.type() === "error") errors.push(`console.error: ${m.text()}`);
});
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
page.on("requestfailed", (r) => net.push(`FAIL ${r.failure()?.errorText} ${r.url()}`));
page.on("response", (r) => {
  if (r.status() >= 400) net.push(`HTTP ${r.status()} ${r.url()}`);
});

async function blur() {
  await page.evaluate(() => (document.activeElement instanceof HTMLElement ? document.activeElement.blur() : null));
}
async function key(code) {
  await blur();
  await page.keyboard.press(code);
}

const steps = [];
async function snap(name) {
  await page.screenshot({ path: `${OUT}-${name}.png` });
  const hud = await page.evaluate(() => {
    const t = document.body.innerText;
    const g = (re) => (t.match(re)?.[0] ?? "").replace(/\s+/g, " ").trim();
    return {
      scenario: g(/Inner System \(SI\)|Apollo 11|OSIRIS-REx/),
      met: g(/MET[^\n]*/),
      warp: g(/[×x]\d[\dkM.]*/),
      paused: t.includes("Paused"),
      velocity: g(/Velocity\s*[\d.]+ km\/s/),
      soi: g(/SOI\s*\n?\s*\w+/),
      nodeUt: g(/Node UT\s*[^\n]+/),
      totalDv: g(/TOTAL ΔV\s*[\d.]+ m\/s/),
      planned: g(/MN\d[\s\S]{0,40}?m\/s/g),
    };
  });
  steps.push({ name, hud });
  console.log(`\n## ${name}\n` + JSON.stringify(hud));
}

await page.goto(URL, { waitUntil: "networkidle", timeout: 60000 });
await page.waitForTimeout(3500);
await snap("00-load");

// 1) Create node + prograde burn.
await page.click('button:has-text("Create node")').catch((e) => errors.push(`create: ${e}`));
await page.waitForTimeout(800);
const prograde = page.locator('input[type="number"]').first();
await prograde.fill("1200");
await prograde.press("Enter");
await page.waitForTimeout(1200);
await snap("01-node-prograde");

// 2) Execute: play + warp, wait for the ~1-day node to fire.
await key("Space"); // play
await page.waitForTimeout(200);
for (let i = 0; i < 12; i++) await page.keyboard.press("Period"); // max warp
await page.waitForTimeout(7000);
await key("Space"); // pause
await page.waitForTimeout(800);
await snap("02-executed");

// 3) Plenty-of-warp sanity already done; now Hohmann pair (Shift+H).
await blur();
await page.keyboard.down("Shift");
await page.keyboard.press("KeyH");
await page.keyboard.up("Shift");
await page.waitForTimeout(1200);
await snap("03-hohmann-pair");

// 4) Cycle vessel (Tab) and clear nodes (C).
await key("KeyC");
await page.waitForTimeout(600);
await snap("04-cleared");

// 5) Click the vessel orbit to place a node.
await page.click('button:has-text("Create node")').catch(() => {});
await page.waitForTimeout(600);
const box = await page.locator("canvas").boundingBox();
// sweep a few points to try to hit the (pickable) orbit line
for (const [fx, fy] of [[0.3, 0.4], [0.7, 0.35], [0.5, 0.7], [0.2, 0.55]]) {
  await page.mouse.click(box.x + box.width * fx, box.y + box.height * fy);
  await page.waitForTimeout(500);
}
await snap("05-orbit-click");

// 6) Switch scenarios.
for (const code of ["Digit2", "Digit3", "Digit1"]) {
  await key(code);
  await page.waitForTimeout(2800);
  await snap(`06-scenario-${code}`);
}

// 7) Reset.
await key("KeyR");
await page.waitForTimeout(1500);
await snap("07-reset");

console.log("\n===== ERRORS =====");
console.log(errors.length ? [...new Set(errors)].join("\n") : "(none)");
console.log("===== NETWORK FAILURES =====");
console.log(net.length ? [...new Set(net)].join("\n") : "(none)");
await browser.close();
