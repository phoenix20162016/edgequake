import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import en from "../en.json";
import fr from "../fr.json";
import zh from "../zh.json";

type Tree = { [key: string]: string | Tree };

function leaves(tree: Tree, prefix = ""): string[] {
  return Object.entries(tree).flatMap(([key, value]) =>
    typeof value === "string" ? [`${prefix}${key}`] : leaves(value, `${prefix}${key}.`),
  );
}

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (name === "__tests__" || name === "node_modules") return [];
    if (statSync(path).isDirectory()) return sourceFiles(path);
    return /\.(ts|tsx)$/.test(name) ? [path] : [];
  });
}

const root = join(__dirname, "..", "..");
const enKeys = new Set(leaves((en as { extractionMode: Tree }).extractionMode, "extractionMode."));

describe("extractionMode locales (V03)", () => {
  it("has the same keys in en, fr and zh", () => {
    const sorted = (tree: Tree) => leaves(tree, "extractionMode.").sort();
    expect(sorted((fr as { extractionMode: Tree }).extractionMode)).toEqual([...enKeys].sort());
    expect(sorted((zh as { extractionMode: Tree }).extractionMode)).toEqual([...enKeys].sort());
  });

  it("keeps the same {{placeholders}} in every language", () => {
    const placeholders = (text: string) => [...text.matchAll(/\{\{(\w+)\}\}/g)].map((m) => m[1]).sort();
    const walk = (a: Tree, b: Tree, path: string) => {
      for (const [key, value] of Object.entries(a)) {
        const other = b[key];
        if (typeof value === "string") {
          expect(placeholders(other as string), `${path}${key}`).toEqual(placeholders(value));
        } else {
          walk(value, other as Tree, `${path}${key}.`);
        }
      }
    };
    walk((en as { extractionMode: Tree }).extractionMode, (fr as { extractionMode: Tree }).extractionMode, "fr:");
    walk((en as { extractionMode: Tree }).extractionMode, (zh as { extractionMode: Tree }).extractionMode, "zh:");
  });

  it("defines every literal extractionMode key used in the source", () => {
    const used = new Set<string>();
    for (const file of sourceFiles(root)) {
      const text = readFileSync(file, "utf8");
      for (const match of text.matchAll(/\bt\(\s*['"`](extractionMode\.[\w.]+)['"`]/g)) used.add(match[1]);
    }
    expect(used.size).toBeGreaterThan(20);
    for (const key of used) expect(enKeys.has(key), key).toBe(true);
  });

  it("defines every dynamic key family the components build", () => {
    const reasons = ["disabled", "settings_error", "unreachable", "model_missing", "not_decision_capable", "unsupported"];
    for (const reason of reasons) expect(enKeys.has(`extractionMode.status.${reason}`), reason).toBe(true);
    for (const word of ["strict", "balanced", "recall"]) {
      expect(enKeys.has(`extractionMode.preset.${word}`)).toBe(true);
      expect(enKeys.has(`extractionMode.presetHint.${word}`)).toBe(true);
    }
    for (const source of ["document", "workspace", "env", "default"]) {
      expect(enKeys.has(`extractionMode.source.${source}`)).toBe(true);
    }
    for (const mode of ["llm", "decision"]) expect(enKeys.has(`extractionMode.name.${mode}`)).toBe(true);
  });
});
