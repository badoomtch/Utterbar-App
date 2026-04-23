#!/usr/bin/env node
// Utterbar version bumper.
//
// Keeps package.json, src-tauri/tauri.conf.json, and src-tauri/Cargo.toml in
// lockstep so we never ship a build whose installer filename, app.getVersion(),
// and crate version disagree. Tauri enforces a single semver across all three
// at build time; this script removes the "did I remember to bump all three"
// ritual.
//
// Usage:
//   node scripts/bump-version.mjs                 # bumps patch (0.3.10 → 0.3.11)
//   node scripts/bump-version.mjs minor           # 0.3.10 → 0.4.0
//   node scripts/bump-version.mjs major           # 0.3.10 → 1.0.0
//   node scripts/bump-version.mjs 0.4.2           # explicit set
//
// Called automatically via `npm run tauri:build:bumped`.

import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');

const pkgPath = resolve(root, 'package.json');
const confPath = resolve(root, 'src-tauri', 'tauri.conf.json');
const cargoPath = resolve(root, 'src-tauri', 'Cargo.toml');

function currentVersion() {
  const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'));
  return pkg.version;
}

function parse(v) {
  const m = /^(\d+)\.(\d+)\.(\d+)$/.exec(v);
  if (!m) throw new Error(`not a semver: ${v}`);
  return [Number(m[1]), Number(m[2]), Number(m[3])];
}

function compute(next) {
  const cur = currentVersion();
  if (!next || next === 'patch') {
    const [a, b, c] = parse(cur);
    return `${a}.${b}.${c + 1}`;
  }
  if (next === 'minor') {
    const [a, b] = parse(cur);
    return `${a}.${b + 1}.0`;
  }
  if (next === 'major') {
    const [a] = parse(cur);
    return `${a + 1}.0.0`;
  }
  // explicit set
  parse(next); // validate
  return next;
}

function bumpJson(path, newVersion) {
  const raw = readFileSync(path, 'utf8');
  const updated = raw.replace(/"version"\s*:\s*"[^"]+"/, `"version": "${newVersion}"`);
  writeFileSync(path, updated);
}

function bumpCargo(path, newVersion) {
  const raw = readFileSync(path, 'utf8');
  // Match the *first* `version = "..."` under [package] only. We anchor on the
  // line prefix so we don't accidentally rewrite a dep's version pin later in
  // the file.
  const updated = raw.replace(/^version\s*=\s*"[^"]+"/m, `version = "${newVersion}"`);
  writeFileSync(path, updated);
}

const arg = process.argv[2];
const oldVersion = currentVersion();
const newVersion = compute(arg);

if (oldVersion === newVersion) {
  console.log(`version unchanged: ${oldVersion}`);
  process.exit(0);
}

bumpJson(pkgPath, newVersion);
bumpJson(confPath, newVersion);
bumpCargo(cargoPath, newVersion);

console.log(`bumped: ${oldVersion} → ${newVersion}`);
