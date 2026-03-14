export type FrontmatterDelimiter = '+++' | '---';

export interface FrontmatterModel {
    frontmatter: string | null;
    body: string;
    delimiter: FrontmatterDelimiter | null;
}

export interface FrontmatterSummary {
    title: string;
    status: string;
    tags: string[];
    specs: string[];
    version?: string;
    target_date?: string;
}

export interface FrontmatterEntry {
    key: string;
    value: string;
}

const FRONTMATTER_RE =
    /^[\s\uFEFF\xA0]*(\+\+\+|---)[ \t]*\r?\n([\s\S]*?)\r?\n[ \t]*\1[ \t]*(?:\r?\n|$)/;

const TAGS_KEY = 'tags';
const SPECS_KEY = 'specs';

function escapeRegex(value: string): string {
    return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function unquote(value: string): string {
    const trimmed = value.trim();
    if ((trimmed.startsWith('"') && trimmed.endsWith('"')) || (trimmed.startsWith("'") && trimmed.endsWith("'"))) {
        return trimmed.slice(1, -1).replace(/\\"/g, '"').replace(/\\'/g, "'").replace(/\\\\/g, '\\');
    }
    return trimmed;
}

function quote(value: string): string {
    return `"${value.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`;
}

function parseInlineList(raw: string): string[] {
    const trimmed = raw.trim();
    if (!trimmed.startsWith('[') || !trimmed.endsWith(']')) return [];

    const inner = trimmed.slice(1, -1).trim();
    if (!inner) return [];

    return inner
        .split(',')
        .map((item) => unquote(item))
        .map((item) => item.trim())
        .filter(Boolean);
}

function normalizeFrontmatter(frontmatter: string | null): string {
    return (frontmatter ?? '').trim();
}

function upsertLine(
    frontmatter: string | null,
    key: string,
    value: string,
    delimiter: FrontmatterDelimiter
): string {
    const current = normalizeFrontmatter(frontmatter);
    const line = delimiter === '+++' ? `${key} = ${quote(value)}` : `${key}: ${quote(value)}`;
    const pattern = new RegExp(`^${escapeRegex(key)}\\s*(?:=|:)\\s*.*$`, 'm');

    if (!current) return line;
    if (pattern.test(current)) {
        return current.replace(pattern, line);
    }

    return `${current}\n${line}`;
}

function upsertListLine(
    frontmatter: string | null,
    key: string,
    values: string[],
    delimiter: FrontmatterDelimiter
): string {
    const current = normalizeFrontmatter(frontmatter);
    const compactValues = Array.from(new Set(values.map((value) => value.trim()).filter(Boolean)));
    const serialized = `[${compactValues.map((value) => quote(value)).join(', ')}]`;
    const line = delimiter === '+++' ? `${key} = ${serialized}` : `${key}: ${serialized}`;
    const pattern = new RegExp(`^${escapeRegex(key)}\\s*(?:=|:)\\s*.*$`, 'm');

    if (!current) return line;
    if (pattern.test(current)) {
        return current.replace(pattern, line);
    }

    return `${current}\n${line}`;
}

function removeLine(frontmatter: string | null, key: string): string | null {
    const current = normalizeFrontmatter(frontmatter);
    if (!current) return null;

    const pattern = new RegExp(`^${escapeRegex(key)}\\s*(?:=|:)\\s*.*$(?:\\r?\\n)?`, 'gm');
    const next = current.replace(pattern, '').replace(/\n{3,}/g, '\n\n').trim();
    return next || null;
}

export function splitFrontmatterDocument(markdown: string): FrontmatterModel {
    const match = markdown.match(FRONTMATTER_RE);
    if (!match) {
        return {
            frontmatter: null,
            body: markdown,
            delimiter: null,
        };
    }
    const delimiter = match[1] as FrontmatterDelimiter;
    const frontmatter = match[2].trim();
    return {
        frontmatter: frontmatter || null,
        body: stripAllFrontmatter(markdown.slice(match[0].length)),
        delimiter,
    };
}

export function stripAllFrontmatter(markdown: string): string {
    let current = markdown;
    while (true) {
        const match = current.match(FRONTMATTER_RE);
        if (!match) break;
        current = current.slice(match[0].length).replace(/^\n+/, '');
    }
    return current;
}

export function composeFrontmatterDocument(
    frontmatter: string | null,
    body: string,
    delimiter: FrontmatterDelimiter = '+++'
): string {
    const cleanedFrontmatter = normalizeFrontmatter(frontmatter);
    if (!cleanedFrontmatter) return body;

    const cleanedBody = body.replace(/^\n+/, '');
    return `${delimiter}\n${cleanedFrontmatter}\n${delimiter}${cleanedBody ? `\n\n${cleanedBody}` : '\n'}`;
}

export function parseFrontmatterEntries(frontmatter: string | null): FrontmatterEntry[] {
    const current = normalizeFrontmatter(frontmatter);
    if (!current) return [];

    return current
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter((line) => line.length > 0 && !line.startsWith('#'))
        .map((line) => {
            const match = line.match(/^([A-Za-z0-9_.-]+)\s*(?:=|:)\s*(.*)$/);
            if (!match) return null;
            return {
                key: match[1],
                value: match[2].trim(),
            };
        })
        .filter((entry): entry is FrontmatterEntry => entry !== null);
}

function findLineValue(frontmatter: string | null, key: string): string | null {
    const current = normalizeFrontmatter(frontmatter);
    if (!current) return null;

    const pattern = new RegExp(`^${escapeRegex(key)}\\s*(?:=|:)\\s*(.*)$`, 'm');
    const match = current.match(pattern);
    return match?.[1]?.trim() ?? null;
}

function parseBoolean(raw: string): boolean | null {
    const normalized = raw.trim().toLowerCase();
    if (normalized === 'true') return true;
    if (normalized === 'false') return false;
    return null;
}

export function readFrontmatterSummary(frontmatter: string | null): FrontmatterSummary {
    const rawTitle = findLineValue(frontmatter, 'title');
    const rawStatus = findLineValue(frontmatter, 'status');
    const rawTags = findLineValue(frontmatter, TAGS_KEY);
    const rawSpecs = findLineValue(frontmatter, SPECS_KEY);
    const rawVersion = findLineValue(frontmatter, 'version');
    const rawTargetDate = findLineValue(frontmatter, 'target_date');

    return {
        title: rawTitle ? unquote(rawTitle) : '',
        status: rawStatus ? unquote(rawStatus) : '',
        tags: rawTags ? parseInlineList(rawTags) : [],
        specs: rawSpecs ? parseInlineList(rawSpecs) : [],
        version: rawVersion ? unquote(rawVersion) : undefined,
        target_date: rawTargetDate ? unquote(rawTargetDate) : undefined,
    };
}

export function readFrontmatterStringField(frontmatter: string | null, key: string): string {
    const rawValue = findLineValue(frontmatter, key);
    return rawValue ? unquote(rawValue) : '';
}

export function readFrontmatterBooleanField(frontmatter: string | null, key: string): boolean | null {
    const rawValue = findLineValue(frontmatter, key);
    if (!rawValue) return null;
    return parseBoolean(rawValue);
}

export function readFrontmatterStringListField(frontmatter: string | null, key: string): string[] {
    const rawValue = findLineValue(frontmatter, key);
    if (!rawValue) return [];
    return parseInlineList(rawValue);
}

export function setFrontmatterStringField(
    frontmatter: string | null,
    key: string,
    value: string,
    delimiter: FrontmatterDelimiter
): string | null {
    const cleanValue = value.trim();
    if (!cleanValue) return removeLine(frontmatter, key);
    return upsertLine(frontmatter, key, cleanValue, delimiter);
}

export function setFrontmatterBooleanField(
    frontmatter: string | null,
    key: string,
    value: boolean | null,
    delimiter: FrontmatterDelimiter
): string | null {
    if (value === null) return removeLine(frontmatter, key);
    const current = normalizeFrontmatter(frontmatter);
    const literal = value ? 'true' : 'false';
    const line = delimiter === '+++' ? `${key} = ${literal}` : `${key}: ${literal}`;
    const pattern = new RegExp(`^${escapeRegex(key)}\\s*(?:=|:)\\s*.*$`, 'm');

    if (!current) return line;
    if (pattern.test(current)) {
        return current.replace(pattern, line);
    }

    return `${current}\n${line}`;
}

export function setFrontmatterStringListField(
    frontmatter: string | null,
    key: string,
    values: string[],
    delimiter: FrontmatterDelimiter
): string | null {
    const compactValues = Array.from(new Set(values.map((value) => value.trim()).filter(Boolean)));
    if (compactValues.length === 0) return removeLine(frontmatter, key);
    return upsertListLine(frontmatter, key, compactValues, delimiter);
}
