/**
 * collect-skill-docs.ts
 *
 * Collects .md/.mdoc files from .ship/skills/<id>/references/docs/ into
 * src/content/docs/skills/<id>/ with Starlight frontmatter. Filters by
 * audience, supports order and section fields. Run as prebuild/predev via tsx.
 */
import { readFileSync, writeFileSync, mkdirSync, existsSync, readdirSync, rmSync } from "fs";
import { join, resolve, dirname, basename, extname } from "path";

const SCRIPT_DIR = dirname(new URL(import.meta.url).pathname);
const REPO_ROOT = resolve(SCRIPT_DIR, "../../../");
const SKILLS_DIR = join(REPO_ROOT, ".ship/skills");
const OUTPUT_DIR = join(SCRIPT_DIR, "../src/content/docs/skills");
const SKIP_AUDIENCES = new Set(["internal", "agent-only"]);

interface Frontmatter { [key: string]: string | null }

function extractField(block: string, field: string): string | null {
  const m = block.match(new RegExp(`^${field}:\\s*(.+)$`, "m"));
  return m ? m[1].trim().replace(/^["']|["']$/g, "") : null;
}

function parseFrontmatter(text: string): Frontmatter {
  const m = text.match(/^---\n([\s\S]*?)\n---/);
  if (!m) return {};
  const b = m[1];
  return {
    title: extractField(b, "title"),
    name: extractField(b, "name"),
    description: extractField(b, "description"),
    audience: extractField(b, "audience"),
    order: extractField(b, "order"),
    section: extractField(b, "section"),
  };
}

function stripFrontmatter(text: string): string {
  return text.replace(/^---\n[\s\S]*?\n---\n*/, "").trimStart();
}

function esc(s: string): string {
  return `"${s.replace(/"/g, '\\"')}"`;
}

interface StarlightMeta {
  title: string;
  description: string | null;
  sidebarLabel: string | null;
  order: string | null;
  section: string | null;
}

function buildFrontmatter(meta: StarlightMeta): string {
  const lines = ["---", `title: ${esc(meta.title)}`];
  if (meta.description) lines.push(`description: ${esc(meta.description)}`);
  if (meta.sidebarLabel || meta.order || meta.section) {
    lines.push("sidebar:");
    const label = meta.section && meta.sidebarLabel
      ? `[${meta.section}] ${meta.sidebarLabel}`
      : meta.sidebarLabel;
    if (label) lines.push(`  label: ${esc(label)}`);
    if (meta.order) lines.push(`  order: ${meta.order}`);
  }
  lines.push("---", "");
  return lines.join("\n");
}

function collectDocsForSkill(skillId: string, docsDir: string): string[] {
  const docFiles = readdirSync(docsDir, { withFileTypes: true })
    .filter((e) => e.isFile() && /\.(md|mdoc)$/.test(e.name))
    .map((e) => e.name);

  const skillMdPath = join(SKILLS_DIR, skillId, "SKILL.md");
  const skillMeta = existsSync(skillMdPath)
    ? parseFrontmatter(readFileSync(skillMdPath, "utf8"))
    : {};
  const written: string[] = [];

  for (const file of docFiles) {
    const text = readFileSync(join(docsDir, file), "utf8");
    const meta = parseFrontmatter(text);
    if (SKIP_AUDIENCES.has(meta.audience ?? "public")) continue;

    const title = meta.title ?? meta.name ?? skillMeta.name
      ?? skillId.charAt(0).toUpperCase() + skillId.slice(1);
    const isIndex = basename(file, extname(file)) === "index";
    const sidebarLabel = isIndex
      ? (skillMeta.name ?? meta.title ?? meta.name ?? null)
      : (meta.title ?? meta.name ?? null);

    const fm = buildFrontmatter({
      title,
      description: meta.description ?? skillMeta.description ?? null,
      sidebarLabel,
      order: meta.order ?? null,
      section: meta.section ?? null,
    });

    const outDir = join(OUTPUT_DIR, skillId);
    mkdirSync(outDir, { recursive: true });
    writeFileSync(join(outDir, file), fm + stripFrontmatter(text), "utf8");
    written.push(file);
  }
  return written;
}

function run(): void {
  if (!existsSync(SKILLS_DIR)) {
    console.log(`[collect-skill-docs] No skills directory at ${SKILLS_DIR}, skipping.`);
    return;
  }

  if (existsSync(OUTPUT_DIR)) {
    rmSync(OUTPUT_DIR, { recursive: true, force: true });
    console.log("[collect-skill-docs] Cleaned output directory.");
  }

  const skillDirs = readdirSync(SKILLS_DIR, { withFileTypes: true })
    .filter((d) => d.isDirectory())
    .map((d) => d.name);

  let total = 0;
  for (const skillId of skillDirs) {
    const docsDir = join(SKILLS_DIR, skillId, "references/docs");
    if (!existsSync(docsDir)) continue;

    const written = collectDocsForSkill(skillId, docsDir);
    if (written.length === 0) continue;

    total += written.length;
    console.log(`[collect-skill-docs] ${skillId} -> ${written.length} file(s): ${written.join(", ")}`);
  }
  console.log(`[collect-skill-docs] Collected ${total} file(s) total.`);
}

run();
