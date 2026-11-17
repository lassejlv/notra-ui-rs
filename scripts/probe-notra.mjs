#!/usr/bin/env node
// Reference probe for porting ui.usenotra.com components 1:1.
//
//   node scripts/probe-notra.mjs styles <path> <selector> [--dark] [--hover] [--depth N]
//   node scripts/probe-notra.mjs shot   <path> <selector> <out.png> [--dark] [--hover] [--scale 2]
//   node scripts/probe-notra.mjs frames <path> <selector> <outdir> [--dark] [--click <sel>|--hover-on <sel>] [--every 33] [--for 1200]
//   node scripts/probe-notra.mjs html   <path> <selector>
//
// <path> is relative to https://ui.usenotra.com (e.g. /components/button). Needs
// playwright-core (npm i playwright-core) and Chrome on PATH or CHROME=/path.
import { mkdirSync } from "node:fs";
import { chromium } from "playwright-core";

const [cmd, path, selector, ...rest] = process.argv.slice(2);
const flag = (name) => rest.includes(`--${name}`);
const opt = (name, fallback) => {
  const i = rest.indexOf(`--${name}`);
  return i >= 0 ? rest[i + 1] : fallback;
};
const PROPS = [
  "display", "position", "width", "height", "padding", "margin", "gap", "border", "border-radius",
  "corner-shape", "background-color", "background-image", "color", "opacity", "box-shadow", "filter",
  "font-family", "font-size", "font-weight", "line-height", "letter-spacing", "text-transform",
  "transform", "transition", "animation", "outline", "align-items", "justify-content", "flex-direction",
  "overflow", "backdrop-filter", "z-index",
];

const browser = await chromium.launch({
  executablePath: process.env.CHROME || "/usr/bin/google-chrome-stable",
  args: ["--force-color-profile=srgb", "--font-render-hinting=none"],
});
const page = await browser.newPage({
  viewport: { width: Number(opt("width", "1280")), height: Number(opt("height", "900")) },
  deviceScaleFactor: Number(opt("scale", "2")),
  colorScheme: flag("dark") ? "dark" : "light",
});
await page.goto(new URL(path, "https://ui.usenotra.com").href, { waitUntil: "load" });
await page.waitForTimeout(800);
const target = page.locator(selector).first();
await target.scrollIntoViewIfNeeded().catch(() => {});
if (flag("hover")) await target.hover();

if (cmd === "styles") {
  const depth = Number(opt("depth", "3"));
  const out = await target.evaluate((el, [props, depth]) => {
    const walk = (node, d) => {
      const cs = getComputedStyle(node);
      const r = node.getBoundingClientRect();
      const style = {};
      for (const p of props) {
        const v = cs.getPropertyValue(p);
        if (v && v !== "none" && v !== "normal" && v !== "auto" && v !== "0px" && v !== "rgba(0, 0, 0, 0)") style[p] = v;
      }
      return {
        tag: node.tagName.toLowerCase(),
        slot: node.getAttribute("data-slot") || undefined,
        class: node.getAttribute("class") || undefined,
        text: node.childNodes.length === 1 && node.firstChild.nodeType === 3 ? node.textContent : undefined,
        rect: [Math.round(r.x), Math.round(r.y), Math.round(r.width * 10) / 10, Math.round(r.height * 10) / 10],
        style,
        children: d > 0 ? [...node.children].map((c) => walk(c, d - 1)) : undefined,
      };
    };
    return walk(el, depth);
  }, [PROPS, depth]);
  console.log(JSON.stringify(out, null, 1));
} else if (cmd === "shot") {
  await target.screenshot({ path: rest[0], animations: "allow" });
  console.log(rest[0]);
} else if (cmd === "html") {
  console.log(await target.evaluate((el) => el.outerHTML));
} else if (cmd === "frames") {
  const dir = rest[0];
  mkdirSync(dir, { recursive: true });
  const every = Number(opt("every", "33"));
  const total = Number(opt("for", "1200"));
  const click = opt("click");
  const hoverOn = opt("hover-on");
  if (click) await page.locator(click).first().click({ noWaitAfter: true });
  if (hoverOn) await page.locator(hoverOn).first().hover();
  const box = await target.boundingBox();
  const pad = Number(opt("pad", "24"));
  const clip = { x: Math.max(0, box.x - pad), y: Math.max(0, box.y - pad), width: box.width + 2 * pad, height: box.height + 2 * pad };
  const start = Date.now();
  let i = 0;
  while (Date.now() - start < total) {
    const t = Date.now() - start;
    await page.screenshot({ path: `${dir}/${String(i).padStart(3, "0")}-${t}ms.png`, clip, animations: "allow" });
    i += 1;
    const next = start + i * every;
    const wait = next - Date.now();
    if (wait > 0) await page.waitForTimeout(wait);
  }
  console.log(`${i} frames in ${dir}`);
} else {
  console.error("unknown command");
  process.exitCode = 2;
}
await browser.close();
