import { getTemplateCmd, type TemplateKind } from '@/lib/platform/tauri/commands';
import { isTauriRuntime } from '@/lib/platform/tauri/runtime';

const TOML_FRONTMATTER_RE = /^\uFEFF?(?:[ \t]*\r?\n)*\+\+\+\r?\n[\s\S]*?\r?\n\+\+\+\r?\n?/;

function escapeRegex(input: string): string {
  return input.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function escapeTomlString(input: string): string {
  return input.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
}

function setTomlStringField(markdown: string, key: string, value: string): string {
  const pattern = new RegExp(`^(${escapeRegex(key)}\\s*=\\s*)\"[^\"]*\"\\s*$`, 'm');
  if (!pattern.test(markdown)) return markdown;
  return markdown.replace(pattern, (_, prefix: string) => `${prefix}"${escapeTomlString(value)}"`);
}

interface LoadTemplateOptions {
  bodyOnly?: boolean;
  tomlValues?: Record<string, string | null | undefined>;
}

export async function loadProjectTemplate(
  kind: TemplateKind,
  options: LoadTemplateOptions = {}
): Promise<string | null> {
  if (!isTauriRuntime()) return null;
  try {
    let template = await getTemplateCmd(kind);
    if (options.tomlValues) {
      for (const [key, value] of Object.entries(options.tomlValues)) {
        if (value === undefined || value === null) continue;
        template = setTomlStringField(template, key, value);
      }
    }
    if (options.bodyOnly) {
      template = template.replace(TOML_FRONTMATTER_RE, '').replace(/^\s+/, '');
    }
    return template;
  } catch {
    return null;
  }
}
