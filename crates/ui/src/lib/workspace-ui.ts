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

const STATUS_COLOR_CLASSES: Record<string, { color: string; bg: string; border: string }> = {
  gray: { color: 'text-zinc-400', bg: 'bg-zinc-800', border: 'border-zinc-700' },
  blue: { color: 'text-blue-400', bg: 'bg-blue-500/10', border: 'border-blue-500/30' },
  red: { color: 'text-red-400', bg: 'bg-red-500/10', border: 'border-red-500/30' },
  green: { color: 'text-emerald-400', bg: 'bg-emerald-500/10', border: 'border-emerald-500/30' },
  yellow: { color: 'text-amber-400', bg: 'bg-amber-500/10', border: 'border-amber-500/30' },
  purple: { color: 'text-violet-400', bg: 'bg-violet-500/10', border: 'border-violet-500/30' },
  orange: { color: 'text-orange-400', bg: 'bg-orange-500/10', border: 'border-orange-500/30' },
};

export function getStatusStyles(status: StatusConfig): {
  label: string;
  color: string;
  bg: string;
  border: string;
} {
  const classes = STATUS_COLOR_CLASSES[status.color ?? 'gray'] ?? STATUS_COLOR_CLASSES.gray;
  return {
    label: status.name,
    color: classes.color,
    bg: classes.bg,
    border: classes.border,
  };
}
