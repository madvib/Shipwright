export const OVERVIEW_ROUTE = '/project/overview' as const;
export const PROJECTS_ROUTE = '/projects' as const;
export const ISSUES_ROUTE = '/project/issues' as const;
export const RELEASES_ROUTE = '/project/releases' as const;
export const FEATURES_ROUTE = '/project/features' as const;
export const SPECS_ROUTE = '/project/specs' as const;
export const ADRS_ROUTE = '/project/adrs' as const;
export const ACTIVITY_ROUTE = '/project/activity' as const;
export const AGENTS_ROUTE = '/project/agents' as const;
export const AGENTS_PROVIDERS_ROUTE = '/project/agents/providers' as const;
export const AGENTS_MCP_ROUTE = '/project/agents/mcp' as const;
export const AGENTS_SKILLS_ROUTE = '/project/agents/skills' as const;
export const AGENTS_PROMPTS_ROUTE = '/project/agents/prompts' as const;
export const SETTINGS_ROUTE = '/project/settings' as const;

export type AppRoutePath =
  | typeof OVERVIEW_ROUTE
  | typeof PROJECTS_ROUTE
  | typeof ISSUES_ROUTE
  | typeof RELEASES_ROUTE
  | typeof FEATURES_ROUTE
  | typeof SPECS_ROUTE
  | typeof ADRS_ROUTE
  | typeof ACTIVITY_ROUTE
  | typeof AGENTS_ROUTE
  | typeof AGENTS_PROVIDERS_ROUTE
  | typeof AGENTS_MCP_ROUTE
  | typeof AGENTS_SKILLS_ROUTE
  | typeof AGENTS_PROMPTS_ROUTE
  | typeof SETTINGS_ROUTE;

export const ROUTE_LABELS: Record<AppRoutePath, string> = {
  [OVERVIEW_ROUTE]: 'Overview',
  [PROJECTS_ROUTE]: 'Projects',
  [ISSUES_ROUTE]: 'Issues',
  [RELEASES_ROUTE]: 'Releases',
  [FEATURES_ROUTE]: 'Features',
  [SPECS_ROUTE]: 'Specs',
  [ADRS_ROUTE]: 'Decisions',
  [ACTIVITY_ROUTE]: 'Activity',
  [AGENTS_ROUTE]: 'Agents',
  [AGENTS_PROVIDERS_ROUTE]: 'Agents / Providers',
  [AGENTS_MCP_ROUTE]: 'Agents / MCP',
  [AGENTS_SKILLS_ROUTE]: 'Agents / Skills',
  [AGENTS_PROMPTS_ROUTE]: 'Agents / Prompts',
  [SETTINGS_ROUTE]: 'Settings',
};

export function normalizePath(pathname: string): string {
  if (pathname.length > 1 && pathname.endsWith('/')) {
    return pathname.slice(0, -1);
  }
  return pathname;
}
