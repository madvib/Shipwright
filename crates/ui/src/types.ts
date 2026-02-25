export interface Project {
    name: string;
    path: string;
    issue_count?: number;
}

export interface StatusConfig {
    id: string;
    name: string;
    color: string;
}

export interface AiConfig {
    provider?: string | null;
    model?: string | null;
    cli_path?: string | null;
}

export interface AgentLayerConfig {
    skills: string[];
    prompts: string[];
    context: string[];
    rules: string[];
}

export interface ModeConfig {
    id: string;
    name: string;
    description?: string | null;
    active_tools: string[];
    mcp_servers: string[];
}

export interface McpServerConfig {
    id: string;
    name: string;
    command: string;
    args: string[];
    env: Record<string, string>;
    scope: 'global' | 'project' | 'mode';
}

export interface ProjectConfig {
    version: string;
    name?: string | null;
    description?: string | null;
    statuses: StatusConfig[];
    git?: GitConfig;
    ai?: AiConfig | null;
    modes?: ModeConfig[];
    mcp_servers?: McpServerConfig[];
    active_mode?: string | null;
    agent?: AgentLayerConfig;
}

export interface GitConfig {
    ignore: string[];
    commit: string[];
}

export const DEFAULT_STATUSES: StatusConfig[] = [
    { id: 'backlog', name: 'Backlog', color: 'gray' },
    { id: 'in-progress', name: 'In Progress', color: 'blue' },
    { id: 'review', name: 'Review', color: 'yellow' },
    { id: 'blocked', name: 'Blocked', color: 'red' },
    { id: 'done', name: 'Done', color: 'green' },
];

export interface IssueMetadata {
    title: string;
    created: string;
    updated: string;
    assignee?: string | null;
    tags: string[];
    spec?: string | null;
    links: string[];
}

export interface Issue {
    metadata: IssueMetadata;
    description: string;
}

export interface IssueEntry {
    file_name: string;
    status: string;
    path: string;
    issue: Issue;
}

export interface AdrMetadata {
    title: string;
    status: string;
    date: string;
    tags: string[];
    spec?: string | null;
}

export interface ADR {
    metadata: AdrMetadata;
    body: string;
}

export interface AdrEntry {
    file_name: string;
    path: string;
    adr: ADR;
}

export interface SpecEntry {
    file_name: string;
    title: string;
    path: string;
}

export interface SpecDocument extends SpecEntry {
    content: string;
}

export interface LogEntry {
    timestamp: string;
    actor: string;
    action: string;
    details: string;
}

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

const STATUS_COLOR_CLASSES: Record<string, { color: string; bg: string; border: string }> = {
    gray: { color: 'text-zinc-400', bg: 'bg-zinc-800', border: 'border-zinc-700' },
    blue: { color: 'text-blue-400', bg: 'bg-blue-500/10', border: 'border-blue-500/30' },
    red: { color: 'text-red-400', bg: 'bg-red-500/10', border: 'border-red-500/30' },
    green: { color: 'text-emerald-400', bg: 'bg-emerald-500/10', border: 'border-emerald-500/30' },
    yellow: { color: 'text-amber-400', bg: 'bg-amber-500/10', border: 'border-amber-500/30' },
    purple: { color: 'text-violet-400', bg: 'bg-violet-500/10', border: 'border-violet-500/30' },
    orange: { color: 'text-orange-400', bg: 'bg-orange-500/10', border: 'border-orange-500/30' },
};

export function getStatusStyles(status: StatusConfig): { label: string; color: string; bg: string; border: string } {
    const classes = STATUS_COLOR_CLASSES[status.color] ?? STATUS_COLOR_CLASSES.gray;
    return {
        label: status.name,
        color: classes.color,
        bg: classes.bg,
        border: classes.border,
    };
}
