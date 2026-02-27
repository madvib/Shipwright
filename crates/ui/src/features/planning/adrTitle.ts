const FRONTMATTER_PATTERN = /^\uFEFF?(?:[ \t]*\r?\n)*---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/;

function extractFrontmatterTitle(markdown: string): string | null {
  const match = markdown.match(FRONTMATTER_PATTERN);
  if (!match) return null;
  const lines = match[1].split(/\r?\n/);
  for (const line of lines) {
    const titleMatch = line.match(/^\s*title\s*:\s*(.+)\s*$/i);
    if (!titleMatch) continue;
    const cleaned = titleMatch[1].trim().replace(/^['"]|['"]$/g, '');
    if (cleaned) return cleaned;
  }
  return null;
}

function stripFrontmatter(markdown: string): string {
  const match = markdown.match(FRONTMATTER_PATTERN);
  if (!match) return markdown;
  return markdown.slice(match[0].length);
}

function normalizeTitleText(raw: string): string {
  return raw
    .replace(/\[(.*?)\]\((.*?)\)/g, '$1')
    .replace(/[`*_~]/g, '')
    .replace(/\s+/g, ' ')
    .trim()
    .slice(0, 120);
}

export function deriveAdrTitleFromMarkdown(markdown: string): string {
  const frontmatterTitle = extractFrontmatterTitle(markdown);
  if (frontmatterTitle) return normalizeTitleText(frontmatterTitle);

  const body = stripFrontmatter(markdown);
  const lines = body.split(/\r?\n/);
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    const headingMatch = trimmed.match(/^#{1,6}\s+(.+)$/);
    const candidate = headingMatch ? headingMatch[1] : trimmed;
    const normalized = normalizeTitleText(candidate);
    if (normalized) return normalized;
  }

  return '';
}
