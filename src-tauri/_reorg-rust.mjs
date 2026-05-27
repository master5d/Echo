// One-off codemod: group flat crate-root modules into platform/ and
// integrations/ folders, then rewrite `crate::<mod>` paths + lib.rs mod decls.
import fs from "node:fs";
import path from "node:path";

const root = process.cwd(); // src-tauri
const SRC = path.join(root, "src");

// group folder -> modules to move into it.
// Only cross-platform, non-cfg-gated OS/UX peripheral modules: every one of
// these compiles on Windows so the reorg is fully validatable here. Domain
// logic, entry points, integrations (llm_client / macOS-only apple_intelligence)
// stay flat at the crate root, which is idiomatic Rust.
const GROUPS = {
  platform: ["overlay", "clipboard", "input", "tray", "tray_i18n",
    "signal_handle", "audio_feedback"],
};

// mod -> group
const modToGroup = {};
for (const [g, mods] of Object.entries(GROUPS)) for (const m of mods) modToGroup[m] = g;

// 1. Move files src/<mod>.rs -> src/<group>/<mod>.rs
for (const [m, g] of Object.entries(modToGroup)) {
  const oldp = path.join(SRC, `${m}.rs`);
  const newDir = path.join(SRC, g);
  const newp = path.join(newDir, `${m}.rs`);
  if (!fs.existsSync(oldp)) { console.error("MISSING", oldp); process.exit(1); }
  fs.mkdirSync(newDir, { recursive: true });
  fs.renameSync(oldp, newp);
}

// 2. Create group module files (preserve original visibility: these mods were
//    `mod` (crate-private) or `pub mod`; keep pub so crate-wide paths resolve).
const PUB_MODS = new Set(["portable"]); // (none of our moved mods were pub except none)
for (const [g, mods] of Object.entries(GROUPS)) {
  const lines = mods.map((m) => `pub mod ${m};`).join("\n") + "\n";
  fs.writeFileSync(path.join(SRC, `${g}.rs`), lines);
}

// 3. Rewrite all .rs files: crate::<mod>  ->  crate::<group>::<mod>
function walk(dir, acc = []) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) walk(p, acc);
    else if (e.name.endsWith(".rs")) acc.push(p);
  }
  return acc;
}
const files = walk(SRC);
let rewrites = 0;
for (const f of files) {
  let s = fs.readFileSync(f, "utf8");
  let orig = s;
  for (const [m, g] of Object.entries(modToGroup)) {
    // crate::<mod> not already followed by another ident char, and not already crate::<group>::
    const re = new RegExp(`crate::${m}\\b`, "g");
    s = s.replace(re, `crate::${g}::${m}`);
  }
  if (s !== orig) { fs.writeFileSync(f, s); rewrites++; }
}

// 4. Fix lib.rs: replace individual `mod <m>;` lines with group decls.
const libp = path.join(SRC, "lib.rs");
let lib = fs.readFileSync(libp, "utf8");
for (const m of Object.keys(modToGroup)) {
  lib = lib.replace(new RegExp(`^\\s*(pub\\s+)?mod ${m};\\s*\\n`, "m"), "");
}
// insert group mod decls (after `mod actions;` for stable placement)
const groupDecls = Object.keys(GROUPS).map((g) => `mod ${g};`).join("\n") + "\n";
if (/^mod actions;/m.test(lib)) {
  lib = lib.replace(/^mod actions;\s*\n/m, (mm) => mm + groupDecls);
} else {
  lib = groupDecls + lib;
}
fs.writeFileSync(libp, lib);

console.log(`moved ${Object.keys(modToGroup).length} modules into ${Object.keys(GROUPS).length} groups; rewrote ${rewrites} files; lib.rs updated`);
