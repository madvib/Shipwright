import { ADR } from '@/bindings';

export function deriveAdrDocTitle(content: string): string {
  const lines = content.split(/\r?\n/);
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    const heading = trimmed.match(/^#{1,6}\s+(.+)$/)?.[1]?.trim() ?? '';
    const candidate = (heading || trimmed).replace(/\s+/g, ' ').trim();
    if (!candidate) continue;
    if (/^decision$/i.test(candidate)) continue;
    return candidate.slice(0, 120);
  }
  return '';
}

export function deriveAdrHeaderTitle(adr: ADR, fallbackFileName: string): string {
  if (adr.metadata.title?.trim()) return adr.metadata.title.trim();
  const decisionTitle = deriveAdrDocTitle(adr.decision);
  if (decisionTitle) return decisionTitle;
  const contextTitle = deriveAdrDocTitle(adr.context);
  if (contextTitle) return contextTitle;
  return fallbackFileName.replace(/\.md$/i, '');
}
