import assert from "node:assert/strict";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { readFile } from "node:fs/promises";

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
for (const [patch, code] of [
  [{ counts34: [-1, ...counts34.slice(1)], unavailable_counts34: [-1, ...counts34.slice(1)] }, "COUNT_OUT_OF_RANGE"],
  [{ meld_count: -1 }, "MELD_COUNT_OUT_OF_RANGE"],
]) {
  try {
    wasm.calculate_shanten34({ ...input, ...patch });
    assert.fail(`expected ${code}`);
  } catch (error) {
    assert.equal(error.code, code);
  }
}

const fixture = JSON.parse(await readFile("fixtures/calc-m0-v1.json", "utf8"));
for (const testCase of fixture.cases) {
  if (testCase.expect_error) {
    try {
      wasm[testCase.operation](testCase.input);
      assert.fail(`${testCase.name}: expected ${testCase.expect_error}`);
    } catch (error) {
      assert.equal(error.code, testCase.expect_error, testCase.name);
    }
  } else {
    assertPartial(testCase.name, wasm[testCase.operation](testCase.input), testCase.expect ?? {});
  }
}

const batch = wasm.calculate_batch34([
  { id: "ok", operation: "analyze_draws34", input },
  { id: "bad", operation: "analyze_draws34", input: { ...input, counts34: [] } },
]);
assert.deepEqual(batch.map((item) => item.id), ["ok", "bad"]);
assert.equal(batch[0].result.kind, "draws");
assert.equal(batch[1].error.code, "HAND_MELD_INCONSISTENT");
try {
  wasm.calculate_batch34([
    { id: "same", operation: "analyze_draws34", input },
    { id: "same", operation: "analyze_draws34", input },
  ]);
  assert.fail("expected duplicate batch id");
} catch (error) {
  assert.equal(error.code, "DUPLICATE_BATCH_ID");
}

function assertPartial(name, result, expected) {
  if (Array.isArray(result)) return;
  for (const [field, expectedValue] of Object.entries(expected)) {
    let actual;
    if (field === "minimum_shanten") actual = (result.shanten ?? result).minimum;
    else if (field.endsWith("_tiles34")) actual = result[field.slice(0, -2)].map((tile) => tile.tile34);
    else actual = result[field];
    assert.deepEqual(actual, expectedValue, `${name}: ${field}`);
  }
}
