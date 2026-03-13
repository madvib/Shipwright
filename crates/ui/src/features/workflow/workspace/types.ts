import { WorkspaceGraphRow, WorkspaceGraphStatus } from '../components/WorkspaceLifecycleGraph';

export interface WorkspaceRow extends WorkspaceGraphRow {
    id: string;
    branch: string;
    environmentId: string | null;
    mcpServers: string[];
    skills: string[];
    resolvedAt: string;
    worktreePath: string | null;
    lastActivatedAt: string | null;
    contextHash: string | null;
    configGeneration: number;
    compiledAt: string | null;
    compileError: string | null;
    status: WorkspaceGraphStatus;
}

export type WorkspaceGroupBy = 'type' | 'release' | 'status';
