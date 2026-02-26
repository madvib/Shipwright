import { ProjectDiscovery } from '@/bindings';

type Project = ProjectDiscovery;

export const THEME_STORAGE_KEY = 'ship-ui-theme';
export const SETTINGS_STORAGE_KEY = 'ship-ui-settings';
export const SIDEBAR_COLLAPSED_STORAGE_KEY = 'ship-ui-sidebar-collapsed';

export const normalizeProjectPath = (rawPath: string): string => {
  const trimmed = rawPath.trim();
  if (trimmed === '/' || /^[A-Za-z]:[\\/]$/.test(trimmed)) {
    return trimmed;
  }
  const collapsed = trimmed.replace(/[\\/]+$/, '');
  const normalized = collapsed || trimmed;
  if (/^[A-Za-z]:[\\/]/.test(normalized)) {
    return normalized.toLowerCase();
  }
  return normalized;
};

export const dedupeProjects = (projects: ProjectDiscovery[]): Project[] => {
  const seen = new Set<string>();
  const deduped: Project[] = [];

  for (const project of projects) {
    const path = normalizeProjectPath(project.path.toString());
    if (!path || seen.has(path)) continue;
    seen.add(path);
    deduped.push({
      name: project.name,
      path,
      issue_count: typeof project.issue_count === 'number' ? project.issue_count : undefined,
    });
  }

  return deduped;
};

export const projectFromInfo = (info: { name: string; path: string; issue_count?: number }): Project => ({
  name: info.name,
  path: normalizeProjectPath(info.path),
  issue_count: info.issue_count,
});

export const applyTheme = (theme?: string) => {
  const resolved = theme === 'light' ? 'light' : 'dark';
  document.documentElement.classList.toggle('dark', resolved === 'dark');
  document.body.dataset.theme = resolved;
};
