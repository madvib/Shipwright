#!/usr/bin/env node
/**
 * collect-skill-docs.mjs
 *
 * Walks .ship/skills/*\/references/docs/index.md, reads SKILL.md frontmatter
 * for metadata, and copies content into src/content/docs/skills/<id>/index.md
 * with correct Starlight frontmatter injected.
 *
 * Run automatically as prebuild / predev.
 */

import { readFileSync, writeFileSync, mkdirSync, existsSync } from "fs";
import { join, resolve, dirname } from "path";
import { fileURLToPath } from "url";
import { readdirSync } from "fs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(__dirname, "../../../");
const SKILLS_DIR = join(REPO_ROOT, ".ship/skills");
const OUTPUT_DIR = join(__dirname, "../src/content/docs/skills");

/** Extract a frontmatter field value from a YAML frontmatter block. */
function extractFrontmatterField(text, field) {
  const match = text.match(new RegExp(`^${field}:\\s*(.+)$`, "m"));
  return match ? match[1].trim().replace(/^["']|["']$/g, "") : null;
}

/** Parse the YAML frontmatter block from a file's text content. */
function parseFrontmatter(text) {
  const match = text.match(/^---\n([\s\S]*?)\n---/);
  if (!match) return {};
  const block = match[1];
  return {
    title: extractFrontmatterField(block, "title"),
    name: extractFrontmatterField(block, "name"),
    description: extractFrontmatterField(block, "description"),
    stableId: extractFrontmatterField(block, "stable-id"),
  };
}

/** Strip existing frontmatter from markdown text. */
function stripFrontmatter(text) {
  return text.replace(/^---\n[\s\S]*?\n---\n*/, "").trimStart();
}

/** Build Starlight-compatible frontmatter. */
function buildFrontmatter({ title, description, sidebarLabel }) {
  const lines = ["---", `title: "${title.replace(/"/g, '\\"')}"`];
  if (description) {
    lines.push(`description: "${description.replace(/"/g, '\\"')}"`);
  }
  if (sidebarLabel) {
    lines.push(`sidebar:`);
    lines.push(`  label: "${sidebarLabel.replace(/"/g, '\\"')}"`);
  }
  lines.push("---", "");
  return lines.join("\n");
}

function run() {
  if (!existsSync(SKILLS_DIR)) {
    console.log(`[collect-skill-docs] No skills directory found at ${SKILLS_DIR}, skipping.`);
    return;
  }

  const skillDirs = readdirSync(SKILLS_DIR, { withFileTypes: true })
    .filter((d) => d.isDirectory())
    .map((d) => d.name);

  let collected = 0;

  for (const skillId of skillDirs) {
    const docsIndex = join(SKILLS_DIR, skillId, "references/docs/index.md");
    if (!existsSync(docsIndex)) continue;

    // Read skill docs content
    const docsText = readFileSync(docsIndex, "utf8");

    // Try to get metadata from SKILL.md frontmatter
    const skillMd = join(SKILLS_DIR, skillId, "SKILL.md");
    let skillMeta = {};
    if (existsSync(skillMd)) {
      skillMeta = parseFrontmatter(readFileSync(skillMd, "utf8"));
    }

    // Try to get title/description from docs frontmatter first, fall back to SKILL.md
    const docsMeta = parseFrontmatter(docsText);
    const title =
      docsMeta.title ||
      docsMeta.name ||
      skillMeta.name ||
      skillId.charAt(0).toUpperCase() + skillId.slice(1);
    const description = docsMeta.description || skillMeta.description || null;
    const sidebarLabel = skillMeta.name || docsMeta.title || docsMeta.name || null;

    const body = stripFrontmatter(docsText);
    const frontmatter = buildFrontmatter({ title, description, sidebarLabel });
    const output = frontmatter + body;

    const outDir = join(OUTPUT_DIR, skillId);
    mkdirSync(outDir, { recursive: true });
    const outFile = join(outDir, "index.md");
    writeFileSync(outFile, output, "utf8");

    console.log(`[collect-skill-docs] ${skillId} → src/content/docs/skills/${skillId}/index.md`);
    collected++;
  }

  console.log(`[collect-skill-docs] Collected ${collected} skill(s).`);
}

run();
