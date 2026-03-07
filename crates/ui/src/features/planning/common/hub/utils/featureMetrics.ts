export interface ChecklistSummary {
  total: number;
  done: number;
  open: number;
  percent: number;
}

export interface FeatureChecklistMetrics {
  todos: ChecklistSummary;
  acceptance: ChecklistSummary;
  readinessPercent: number;
  blocking: boolean;
}

function normalizeHeading(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^\w\s-]/g, '')
    .replace(/\s+/g, ' ');
}

function headingMatch(line: string): { level: number; text: string } | null {
  const match = line.match(/^\s{0,3}(#{2,6})\s+(.+?)\s*$/);
  if (!match) return null;
  return {
    level: match[1].length,
    text: normalizeHeading(match[2]),
  };
}

function findSectionBody(markdown: string, headings: string[]): string {
  const target = new Set(headings.map(normalizeHeading));
  const lines = markdown.split(/\r?\n/);
  let start = -1;
  let level = 0;

  for (let i = 0; i < lines.length; i += 1) {
    const heading = headingMatch(lines[i]);
    if (!heading) continue;
    if (target.has(heading.text)) {
      start = i + 1;
      level = heading.level;
      break;
    }
  }

  if (start < 0) return '';

  let end = lines.length;
  for (let i = start; i < lines.length; i += 1) {
    const heading = headingMatch(lines[i]);
    if (!heading) continue;
    if (heading.level <= level) {
      end = i;
      break;
    }
  }

  return lines.slice(start, end).join('\n');
}

function summarizeChecklist(markdown: string): ChecklistSummary {
  const lines = markdown.split(/\r?\n/);
  let total = 0;
  let done = 0;
  for (const line of lines) {
    const item = line.match(/^\s*[-*]\s+\[( |x|X)\]\s+/);
    if (!item) continue;
    total += 1;
    if (item[1].toLowerCase() === 'x') {
      done += 1;
    }
  }
  const open = Math.max(total - done, 0);
  const percent = total === 0 ? 0 : Math.round((done / total) * 100);
  return { total, done, open, percent };
}

export function featureStatusFallbackReadiness(status: string): number {
  switch (status) {
    case 'implemented':
      return 100;
    case 'in-progress':
      return 55;
    case 'planned':
      return 15;
    case 'deprecated':
      return 100;
    default:
      return 0;
  }
}

export function formatStatusLabel(status: string): string {
  return status
    .split('-')
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ');
}

export function deriveFeatureChecklistMetrics(markdown: string, status: string): FeatureChecklistMetrics {
  const fallback = summarizeChecklist(markdown);
  const todosSection = findSectionBody(markdown, ['delivery todos', 'todos', 'delivery']);
  const acceptanceSection = findSectionBody(markdown, ['acceptance criteria']);
  const todos = summarizeChecklist(todosSection);
  const acceptance = summarizeChecklist(acceptanceSection);

  const resolvedTodos = todos.total > 0 ? todos : fallback;
  const baseReadiness =
    resolvedTodos.total > 0 ? resolvedTodos.percent : featureStatusFallbackReadiness(status);
  const acceptanceReadiness =
    acceptance.total > 0 ? acceptance.percent : featureStatusFallbackReadiness(status);
  const readinessPercent = Math.round(baseReadiness * 0.6 + acceptanceReadiness * 0.4);

  const blocking =
    status !== 'deprecated' &&
    (acceptance.total > 0 ? acceptance.open > 0 : status !== 'implemented');

  return {
    todos: resolvedTodos,
    acceptance,
    readinessPercent,
    blocking,
  };
}
