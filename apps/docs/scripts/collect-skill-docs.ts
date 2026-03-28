/**
 * collect-skill-docs.ts
 *
 * Collects .mdoc files from .ship/skills/<id>/references/docs/ into the Astro
 * content directory. ONLY .mdoc files are collected — .md files are usage notes
 * for agents, not project documentation.
 *
 * Site structure is driven by frontmatter in each .mdoc file:
 *   - title (required): page title
 *   - description: meta description
 *   - audience: public (default) | internal | agent-only
 *   - group: sidebar group (e.g. "CLI", "MCP", "Compiler")
 *   - order: sort position within group
 *
 * Run as prebuild/predev via tsx.
 */
import {
  readFileSync, writeFileSync, mkdirSync, existsSync,
  readdirSync, rmSync,
} from "fs"
import { join, resolve, dirname, basename } from "path"

const SCRIPT_DIR = dirname(new URL(import.meta.url).pathname)
const REPO_ROOT = resolve(SCRIPT_DIR, "../../../")
const SKILLS_DIR = join(REPO_ROOT, ".ship/skills")
const OUTPUT_DIR = join(SCRIPT_DIR, "../src/content/docs")
const GENERATED_DIR = join(OUTPUT_DIR, "reference")
const SKIP_AUDIENCES = new Set(["internal", "agent-only"])

function extractField(block: string, field: string): string | null {
  const m = block.match(new RegExp(`^${field}:\\s*(.+)$`, "m"))
  return m ? m[1].trim().replace(/^["']|["']$/g, "") : null
}

function parseFrontmatter(text: string) {
  const m = text.match(/^---\n([\s\S]*?)\n---/)
  if (!m) return {}
  const b = m[1]
  return {
    title: extractField(b, "title"),
    description: extractField(b, "description"),
    audience: extractField(b, "audience"),
    group: extractField(b, "group"),
    order: extractField(b, "order"),
  }
}

function stripFrontmatter(text: string): string {
  return text.replace(/^---\n[\s\S]*?\n---\n*/, "").trimStart()
}

/** Remove the first h1 if present — Starlight renders its own from title. */
function stripLeadingH1(text: string): string {
  return text.replace(/^#\s+.+\n+/, "")
}

function esc(s: string): string {
  return `"${s.replace(/"/g, '\\"')}"`
}

function buildStarlightFrontmatter(meta: {
  title: string
  description: string | null
  label: string | null
  order: string | null
}): string {
  const lines = ["---", `title: ${esc(meta.title)}`]
  if (meta.description) lines.push(`description: ${esc(meta.description)}`)
  if (meta.label || meta.order) {
    lines.push("sidebar:")
    if (meta.label) lines.push(`  label: ${esc(meta.label)}`)
    if (meta.order) lines.push(`  order: ${meta.order}`)
  }
  lines.push("---", "")
  return lines.join("\n")
}

function run(): void {
  if (!existsSync(SKILLS_DIR)) {
    console.log("[collect-docs] No skills directory, skipping.")
    return
  }

  // Clean generated docs (preserve hand-written pages like index.md)
  if (existsSync(GENERATED_DIR)) {
    rmSync(GENERATED_DIR, { recursive: true, force: true })
  }
  mkdirSync(GENERATED_DIR, { recursive: true })

  const skillDirs = readdirSync(SKILLS_DIR, { withFileTypes: true })
    .filter((d) => d.isDirectory())
    .map((d) => d.name)

  let total = 0

  for (const skillId of skillDirs) {
    const docsDir = join(SKILLS_DIR, skillId, "references/docs")
    if (!existsSync(docsDir)) continue

    // Collect .md and .mdoc files that have a group: field (project docs)
    const allFiles = readdirSync(docsDir, { withFileTypes: true })
      .filter((e) => e.isFile() && /\.(md|mdoc)$/.test(e.name))
      .map((e) => e.name)

    // Only include files with group: frontmatter — that's the doc-skill signal
    const docFiles = allFiles.filter((file) => {
      const text = readFileSync(join(docsDir, file), "utf8")
      const fm = parseFrontmatter(text)
      return fm.group !== null
    })

    if (docFiles.length === 0) continue

    // Read SKILL.md for fallback metadata
    const skillMdPath = join(SKILLS_DIR, skillId, "SKILL.md")
    const skillMeta = existsSync(skillMdPath)
      ? parseFrontmatter(readFileSync(skillMdPath, "utf8"))
      : {}

    for (const file of docFiles) {
      const text = readFileSync(join(docsDir, file), "utf8")
      const meta = parseFrontmatter(text)

      if (SKIP_AUDIENCES.has(meta.audience ?? "public")) continue

      const title = meta.title ?? skillId
      const group = meta.group ?? skillId
      const isIndex = basename(file, ".mdoc") === "index"
      const label = isIndex ? null : (meta.title ?? null)

      // Output path: reference/<group>/<filename>.mdoc
      const groupDir = join(GENERATED_DIR, group.toLowerCase().replace(/\s+/g, "-"))
      mkdirSync(groupDir, { recursive: true })

      const body = stripLeadingH1(stripFrontmatter(text))
      const fm = buildStarlightFrontmatter({
        title,
        description: meta.description ?? null,
        label,
        order: meta.order ?? null,
      })

      writeFileSync(join(groupDir, file), fm + body, "utf8")
      total++
    }

    console.log(`[collect-docs] ${skillId} → ${docFiles.length} page(s)`)
  }

  console.log(`[collect-docs] Total: ${total} pages collected.`)
}

run()
