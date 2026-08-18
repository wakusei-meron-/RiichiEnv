import assert from "node:assert/strict";
import path from "node:path";
import { pathToFileURL } from "node:url";

const pkg = pathToFileURL(path.resolve(process.argv[2], "riichienv_calc_wasm.js"));
const wasm = await import(pkg.href);
const counts34 = Array(34).fill(0);
for (const tile of [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 27, 27]) counts34[tile] += 1;
const input = { variant: "yonma", counts34, meld_count: 0, unavailable_counts34: [...counts34], contract_version: "m0-v1" };
const result = wasm.analyze_draws34(input);
assert.equal(result.shanten.minimum, 0);
assert.deepEqual(result.agari_tiles.map((tile) => tile.tile34), [11]);
assert.equal(result.agari_tile_count, 4);
try {
  wasm.calculate_shanten34({ ...input, contract_version: "wrong" });
  assert.fail("expected typed WASM error");
} catch (error) {
  assert.equal(error.code, "CONTRACT_VERSION_MISMATCH");
}
