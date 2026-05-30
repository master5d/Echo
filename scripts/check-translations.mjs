// scripts/check-translations.mjs — verify every locale matches the en reference.
//
// Plain Node ESM (no bun dependency) so contributors on any package manager
// can run it: `node scripts/check-translations.mjs` (or `npm run check:translations`).

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const LOCALES_DIR = path.join(__dirname, "..", "src", "i18n", "locales");
const REFERENCE_LANG = "en";

function getLanguages() {
  const entries = fs.readdirSync(LOCALES_DIR, { withFileTypes: true });
  return entries
    .filter((entry) => entry.isDirectory() && entry.name !== REFERENCE_LANG)
    .map((entry) => entry.name)
    .sort();
}

const LANGUAGES = getLanguages();

const colors = {
  reset: "\x1b[0m",
  red: "\x1b[31m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
};

function colorize(text, color) {
  return `${colors[color]}${text}${colors.reset}`;
}

function getAllKeyPaths(obj, prefix = []) {
  let paths = [];
  for (const key in obj) {
    if (!Object.hasOwn(obj, key)) continue;
    const currentPath = prefix.concat([key]);
    const value = obj[key];
    if (typeof value === "object" && value !== null && !Array.isArray(value)) {
      paths = paths.concat(getAllKeyPaths(value, currentPath));
    } else {
      paths.push(currentPath);
    }
  }
  return paths;
}

function hasKeyPath(obj, keyPath) {
  let current = obj;
  for (const key of keyPath) {
    if (
      typeof current !== "object" ||
      current === null ||
      current[key] === undefined
    ) {
      return false;
    }
    current = current[key];
  }
  return true;
}

function loadTranslationFile(lang) {
  const filePath = path.join(LOCALES_DIR, lang, "translation.json");
  try {
    return JSON.parse(fs.readFileSync(filePath, "utf8"));
  } catch (error) {
    console.error(colorize(`✗ Error loading ${lang}/translation.json:`, "red"));
    console.error(`  ${error.message}`);
    return null;
  }
}

function validateTranslations() {
  console.log(colorize("\n🌍 Translation Consistency Check\n", "blue"));

  console.log(`Loading reference language: ${REFERENCE_LANG}`);
  const referenceData = loadTranslationFile(REFERENCE_LANG);
  if (!referenceData) {
    console.error(
      colorize(`\n✗ Failed to load reference file (${REFERENCE_LANG})`, "red"),
    );
    process.exit(1);
  }

  const referenceKeyPaths = getAllKeyPaths(referenceData);
  console.log(`Reference has ${referenceKeyPaths.length} keys\n`);

  let hasErrors = false;
  const results = {};

  for (const lang of LANGUAGES) {
    const langData = loadTranslationFile(lang);
    if (!langData) {
      hasErrors = true;
      results[lang] = { valid: false, missing: [], extra: [] };
      continue;
    }
    const missing = referenceKeyPaths.filter(
      (keyPath) => !hasKeyPath(langData, keyPath),
    );
    const langKeyPaths = getAllKeyPaths(langData);
    const extra = langKeyPaths.filter(
      (keyPath) => !hasKeyPath(referenceData, keyPath),
    );
    results[lang] = {
      valid: missing.length === 0 && extra.length === 0,
      missing,
      extra,
    };
    if (missing.length > 0 || extra.length > 0) hasErrors = true;
  }

  console.log(colorize("Results:", "blue"));
  console.log("─".repeat(60));

  for (const lang of LANGUAGES) {
    const result = results[lang];
    if (result.valid) {
      console.log(
        colorize(`✓ ${lang.toUpperCase()}: All keys present`, "green"),
      );
    } else {
      console.log(colorize(`✗ ${lang.toUpperCase()}: Issues found`, "red"));
      if (result.missing.length > 0) {
        console.log(
          colorize(`  Missing ${result.missing.length} keys:`, "yellow"),
        );
        result.missing
          .slice(0, 10)
          .forEach((keyPath) => console.log(`    - ${keyPath.join(".")}`));
        if (result.missing.length > 10) {
          console.log(
            colorize(
              `    ... and ${result.missing.length - 10} more`,
              "yellow",
            ),
          );
        }
      }
      if (result.extra.length > 0) {
        console.log(
          colorize(
            `  Extra ${result.extra.length} keys (not in reference):`,
            "yellow",
          ),
        );
        result.extra
          .slice(0, 10)
          .forEach((keyPath) => console.log(`    - ${keyPath.join(".")}`));
        if (result.extra.length > 10) {
          console.log(
            colorize(`    ... and ${result.extra.length - 10} more`, "yellow"),
          );
        }
      }
      console.log("");
    }
  }

  console.log("─".repeat(60));

  const validCount = Object.values(results).filter((r) => r.valid).length;
  const totalCount = LANGUAGES.length;

  if (hasErrors) {
    console.log(
      colorize(
        `\n✗ Validation failed: ${validCount}/${totalCount} languages passed`,
        "red",
      ),
    );
    process.exit(1);
  } else {
    console.log(
      colorize(
        `\n✓ All ${totalCount} languages have complete translations!`,
        "green",
      ),
    );
    process.exit(0);
  }
}

validateTranslations();
