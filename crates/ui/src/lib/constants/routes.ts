export const OVERVIEW_ROUTE = '/overview' as const;
export const PROJECTS_ROUTE = '/projects' as const;
export const ISSUES_ROUTE = '/issues' as const;
export const SPECS_ROUTE = '/specs' as const;
export const ADRS_ROUTE = '/adrs' as const;
export const ACTIVITY_ROUTE = '/activity' as const;
export const AGENTS_ROUTE = '/agents' as const;
export const SETTINGS_ROUTE = '/settings' as const;

export type AppRoutePath =
  | typeof OVERVIEW_ROUTE
  | typeof PROJECTS_ROUTE
  | typeof ISSUES_ROUTE
  | typeof SPECS_ROUTE
  | typeof ADRS_ROUTE
  | typeof ACTIVITY_ROUTE
  | typeof AGENTS_ROUTE
  | typeof SETTINGS_ROUTE;

export const ROUTE_LABELS: Record<AppRoutePath, string> = {
  [OVERVIEW_ROUTE]: 'Overview',
  [PROJECTS_ROUTE]: 'Projects',
  [ISSUES_ROUTE]: 'Issues',
  [SPECS_ROUTE]: 'Specs',
  [ADRS_ROUTE]: 'Decisions',
  [ACTIVITY_ROUTE]: 'Activity',
  [AGENTS_ROUTE]: 'Agents',
  [SETTINGS_ROUTE]: 'Settings',
};

export function normalizePath(pathname: string): string {
  if (pathname.length > 1 && pathname.endsWith('/')) {
    return pathname.slice(0, -1);
  }
  return pathname;
}
