import { chromium } from "playwright";
import http from "node:http";
import path from "node:path";
import { readFile } from "node:fs/promises";

const root = path.resolve(process.argv[2]);
const server = http.createServer(async (request, response) => {
  const file = request.url === "/" ? "index.html" : request.url.slice(1);
  try {
    if (file.endsWith(".js")) response.setHeader("Content-Type", "text/javascript");
    if (file.endsWith(".wasm")) response.setHeader("Content-Type", "application/wasm");
    if (file.endsWith(".json")) response.setHeader("Content-Type", "application/json");
    const body = file === "index.html"
      ? `<!doctype html><script type="module">import init,* as wasm from './riichienv_calc_wasm.js'; await init(); const fixture=await(await fetch('/fixture.json')).json(); window.outcomes=fixture.cases.map(c=>{try{return{name:c.name,value:wasm[c.operation](c.input)}}catch(error){return{name:c.name,error:{code:error.code}}}});</script>`
      : file === "fixture.json"
        ? await readFile("fixtures/calc-m0-v1.json")
      : await readFile(path.join(root, file));
    response.end(body);
  } catch { response.statusCode = 404; response.end(); }
});
await new Promise((resolve) => server.listen(0, resolve));
const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();
await page.goto(`http://127.0.0.1:${server.address().port}`);
await page.waitForFunction(() => window.outcomes);
const outcomes = await page.evaluate(() => window.outcomes);
const fixture = JSON.parse(await readFile("fixtures/calc-m0-v1.json", "utf8"));
for (const [index, testCase] of fixture.cases.entries()) {
  const outcome = outcomes[index];
  if (testCase.expect_error) {
    if (outcome.error?.code !== testCase.expect_error) process.exitCode = 1;
    continue;
  }
  for (const [field, expected] of Object.entries(testCase.expect ?? {})) {
    const result = outcome.value;
    const actual = field === "minimum_shanten"
      ? (result.shanten ?? result).minimum
      : field.endsWith("_tiles34")
        ? result[field.slice(0, -2)].map((tile) => tile.tile34)
        : result[field];
    if (JSON.stringify(actual) !== JSON.stringify(expected)) process.exitCode = 1;
  }
}
await browser.close();
server.close();
