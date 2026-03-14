import type { StatusConfig } from '@/bindings';

export interface Config {
  theme?: string;
  author?: string;
  email?: string;
  notifications_enabled?: boolean;
  default_status?: string;
  editor?: string;
  mcp_enabled?: boolean;
  mcp_port?: number;
}

export const DEFAULT_STATUSES: StatusConfig[] = [
  { id: 'backlog', name: 'Backlog', color: 'gray' },
  { id: 'in-progress', name: 'In Progress', color: 'blue' },
  { id: 'review', name: 'Review', color: 'yellow' },
  { id: 'blocked', name: 'Blocked', color: 'red' },
  { id: 'done', name: 'Done', color: 'green' },
];

export function getStatusStyles(status: StatusConfig): {
  label: string;
  color: string;
  bg: string;
  border: string;
} {
  const colorMap: Record<string, string> = {
    gray: 'gray',
    blue: 'blue',
    red: 'red',
    green: 'green',
    yellow: 'yellow',
    purple: 'purple',
    orange: 'orange',
  };
  const colorName = colorMap[status.color ?? 'gray'] ?? 'gray';
  
  return {
    label: status.name,
    color: `text-status-${colorName}`,
    bg: `bg-status-${colorName}/10`,
    border: `border-status-${colorName}/30`,
  };
}

const ADR_STATUS_COLORS: Record<string, string> = {
  accepted: 'text-status-green bg-status-green/10',
  rejected: 'text-status-red bg-status-red/10',
  superseded: 'text-status-yellow bg-status-yellow/10',
  proposed: 'text-status-blue bg-status-blue/10',
  deprecated: 'text-muted-foreground bg-muted/60',
};

export function getAdrStatusClasses(status: string): string {
  return ADR_STATUS_COLORS[status] ?? 'text-muted-foreground';
}

export const ADR_STATUS_OPTIONS = [
  { value: 'proposed', label: 'Proposed' },
  { value: 'accepted', label: 'Accepted' },
  { value: 'deprecated', label: 'Deprecated' },
  { value: 'rejected', label: 'Rejected' },
  { value: 'superseded', label: 'Superseded' },
];

export function formatStatusLabel(status: string): string {
  return ADR_STATUS_OPTIONS.find((o) => o.value === status)?.label ?? status;
}
