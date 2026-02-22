export interface Project {
    name: string;
    path: string;
    issue_count?: number;
}

export interface Issue {
    title: string;
    description: string;
    status: string;
    created_at: string;
    updated_at: string;
    links: string[];
}

export interface IssueEntry {
    file_name: string;
    status: string;
    path: string;
    issue: Issue;
}

export interface ADR {
    title: string;
    decision: string;
    status: string;
    date: string;
    links: string[];
}

export interface AdrEntry {
    file_name: string;
    path: string;
    adr: ADR;
}

export interface LogEntry {
    timestamp: string;
    action: string;
    details: string;
}

export interface Config {
    theme?: string;
    author?: string;
    notifications_enabled?: boolean;
    default_status?: string;
}

export type NavSection = 'issues' | 'adrs' | 'log' | 'settings';
export type IssueStatus = 'backlog' | 'in-progress' | 'blocked' | 'done';

export const STATUS_CONFIG: Record<string, { label: string; color: string; bg: string; border: string }> = {
    backlog: { label: 'Backlog', color: 'text-zinc-400', bg: 'bg-zinc-800', border: 'border-zinc-700' },
    'in-progress': { label: 'In Progress', color: 'text-blue-400', bg: 'bg-blue-500/10', border: 'border-blue-500/30' },
    blocked: { label: 'Blocked', color: 'text-red-400', bg: 'bg-red-500/10', border: 'border-red-500/30' },
    done: { label: 'Done', color: 'text-emerald-400', bg: 'bg-emerald-500/10', border: 'border-emerald-500/30' },
};
