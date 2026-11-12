import { readFileSync, readdirSync } from "node:fs";

const dir = new URL("../src/", import.meta.url);
const files = readdirSync(dir).filter((name) => name.endsWith(".tsx"));
const used = new Set();
for (const name of files) {
  const text = readFileSync(new URL(name, dir), "utf8");
  for (const match of text.matchAll(/\bt\(\s*"((?:[^"\\]|\\.)*)"\s*\)/g)) used.add(match[1]);
}
const dictionary = readFileSync(new URL("en.ts", dir), "utf8");
const known = new Set([...dictionary.matchAll(/^ {2}"((?:[^"\\]|\\.)*)":/gm)].map((match) => match[1]));
const missing = [...used].filter((key) => !known.has(key));
if (missing.length > 0) {
  console.error(`Faltam ${missing.length} traduções:\n${missing.join("\n")}`);
  process.exit(1);
}
console.log(`${used.size} textos com tradução em inglês`);
