import { chromium } from "playwright";
import http from "node:http";
import path from "node:path";
import { readFile } from "node:fs/promises";

const root = path.resolve(process.argv[2]);
const server = http.createServer(async (request, response) => {
  const file = request.url === "/" ? "index.html" : request.url.slice(1);
  try {
    const body = file === "index.html"
      ? `<!doctype html><script type="module">import init,* as wasm from './riichienv_calc_wasm.js'; await init(); const c=Array(34).fill(0); for(const t of [0,1,2,3,4,5,6,7,8,9,10,27,27])c[t]++; const input={variant:'yonma',counts34:c,meld_count:0,unavailable_counts34:[...c],contract_version:'m0-v1'}; window.result=wasm.analyze_draws34(input); try { wasm.calculate_shanten34({...input,contract_version:'wrong'}); } catch(e) { window.error=e; }</script>`
      : await readFile(path.join(root, file));
    response.end(body);
  } catch { response.statusCode = 404; response.end(); }
});
await new Promise((resolve) => server.listen(0, resolve));
const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();
await page.goto(`http://127.0.0.1:${server.address().port}`);
await page.waitForFunction(() => window.result && window.error);
const result = await page.evaluate(() => ({ result: window.result, error: window.error }));
if (result.result.shanten.minimum !== 0 || result.result.agari_tile_count !== 4 || result.error.code !== "CONTRACT_VERSION_MISMATCH") process.exitCode = 1;
await browser.close();
server.close();
