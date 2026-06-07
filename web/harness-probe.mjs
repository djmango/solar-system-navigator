import { chromium } from "playwright";
const URL = process.env.HARNESS_URL ?? "http://127.0.0.1:5173/";
const browser = await chromium.launch({ headless: false, args: ["--ignore-gpu-blocklist", "--enable-gpu", "--no-sandbox"] });
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
const errs = [];
page.on("console", (m) => m.type() === "error" && errs.push(m.text()));
page.on("pageerror", (e) => errs.push(`pageerror ${e.message}`));
await page.goto(URL, { waitUntil: "networkidle" });
await page.waitForTimeout(3500);

const notice = () =>
  page.evaluate(() => {
    const t = document.body.innerText;
    return {
      planned: (t.match(/MN\d[\s\S]{0,40}?m\/s/g) || []).join(" | "),
      hohmann: (t.match(/Hohmann[^\n]*/) || [""])[0].replace(/\s+/g, " ").trim(),
      notice: (t.match(/(placed on orbit|node created|Select a vessel|No Hohmann|Could not[^\n]*)/) || [""])[0],
    };
  });

// Fresh-load Hohmann pair (Scout on a circular heliocentric orbit).
await page.evaluate(() => document.activeElement?.blur?.());
await page.keyboard.down("Shift");
await page.keyboard.press("KeyH");
await page.keyboard.up("Shift");
await page.waitForTimeout(1200);
console.log("after Shift+H:", JSON.stringify(await notice()));
await page.screenshot({ path: "/tmp/probe-hohmann.png" });

// Plain H (departure draft).
await page.keyboard.press("KeyH");
await page.waitForTimeout(800);
console.log("after H:", JSON.stringify(await notice()));

console.log("errors:", errs.length ? errs.join("\n") : "(none)");
await browser.close();
