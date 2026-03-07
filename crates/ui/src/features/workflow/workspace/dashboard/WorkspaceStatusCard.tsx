import {
  Badge,
  Button,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@ship/ui';
import {
  GitBranch,
  Info,
  RefreshCw,
  Settings2,
  Sparkles,
} from 'lucide-react';
import { type WorkspaceRepairReport } from '@/lib/platform/tauri/commands';
import { WorkspaceGraphStatus } from '../../components/WorkspaceLifecycleGraph';
import { WorkspaceRow } from '../types';

interface WorkspaceStatusCardProps {
  detail: WorkspaceRow;
  statusVariant: (status: WorkspaceGraphStatus) => 'default' | 'secondary' | 'outline';
  onSync: () => void;
  syncing: boolean;
  onActivate: () => void;
  activating: boolean;
  onRepair: () => void;
  repairing: boolean;
  lastRepairReport: WorkspaceRepairReport | null;
}

export function WorkspaceStatusCard({
  detail,
  statusVariant,
  onSync,
  syncing,
  onActivate,
  activating,
  onRepair,
  repairing,
  lastRepairReport,
}: WorkspaceStatusCardProps) {
  const compileSummary = detail.compiledAt
    ? new Date(detail.compiledAt).toLocaleString()
    : 'Not compiled yet';

  return (
    <section className="rounded-xl border bg-card p-4 shadow-sm">
      <div className="mb-3 flex items-center justify-between gap-3">
        <div className="min-w-0">
          <h4 className="truncate text-sm font-semibold tracking-tight text-foreground">
            {detail.branch}
          </h4>
          <p className="text-[11px] text-muted-foreground">
            {detail.workspaceType} · {detail.isWorktree ? 'Worktree' : 'Checkout'}
          </p>
        </div>
        <Badge
          variant={statusVariant(detail.status)}
          className="h-6 px-2.5 text-[10px] font-semibold uppercase"
        >
          {detail.status}
        </Badge>
      </div>

      <div className="mb-3 grid grid-cols-1 gap-2 md:grid-cols-2">
        <div className="rounded-lg border bg-muted/20 px-3 py-2">
          <div className="mb-1 flex items-center gap-1.5">
            <p className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
              Active Mode
            </p>
            <Tooltip>
              <TooltipTrigger asChild>
                <Info className="size-2.5 cursor-help text-muted-foreground/40 transition-colors hover:text-muted-foreground" />
              </TooltipTrigger>
              <TooltipContent side="top">
                The operational mode this workspace is optimized for (e.g.
                coding, logic, etc.)
              </TooltipContent>
            </Tooltip>
          </div>
          <div className="flex items-center gap-2">
            <Sparkles className="size-3.5 text-primary" />
            <span className="text-xs font-medium text-foreground">
              {detail.activeMode ?? 'Default'}
            </span>
          </div>
        </div>
        <div className="rounded-lg border bg-muted/20 px-3 py-2">
          <div className="mb-1 flex items-center gap-1.5">
            <p className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
              Last Compile
            </p>
            <Tooltip>
              <TooltipTrigger asChild>
                <Info className="size-2.5 cursor-help text-muted-foreground/40 transition-colors hover:text-muted-foreground" />
              </TooltipTrigger>
              <TooltipContent side="top">
                Compile timestamp and generation for this workspace.
              </TooltipContent>
            </Tooltip>
          </div>
          <div className="flex items-center justify-between gap-3 text-[11px]">
            <span className="truncate text-muted-foreground">{compileSummary}</span>
            <span className="font-mono text-muted-foreground">
              gen {detail.configGeneration}
            </span>
          </div>
        </div>
      </div>

      {detail.compileError && (
        <p className="mb-3 rounded-md border border-status-red/30 bg-status-red/5 px-2 py-1.5 text-[11px] text-status-red">
          compile error: {detail.compileError}
        </p>
      )}

      <div className="mb-3 flex gap-2 border-t pt-3">
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              size="sm"
              variant="outline"
              className="h-8 flex-1 gap-1.5 text-xs font-medium"
              onClick={onSync}
              disabled={syncing}
            >
              {syncing ? (
                <RefreshCw className="size-3 animate-spin" />
              ) : (
                <RefreshCw className="size-3" />
              )}
              Sync
            </Button>
          </TooltipTrigger>
          <TooltipContent>Refresh workspace record from current branch/worktree state.</TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              size="sm"
              className="h-8 flex-1 gap-1.5 text-xs font-semibold"
              onClick={onActivate}
              disabled={activating}
            >
              {activating ? (
                <RefreshCw className="size-3 animate-spin" />
              ) : (
                <GitBranch className="size-3" />
              )}
              Activate
            </Button>
          </TooltipTrigger>
          <TooltipContent>Compile and apply this workspace context as the active branch environment.</TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              size="sm"
              variant="outline"
              className="h-8 flex-1 gap-1.5 text-xs font-medium"
              onClick={onRepair}
              disabled={repairing}
            >
              {repairing ? (
                <RefreshCw className="size-3 animate-spin" />
              ) : (
                <Settings2 className="size-3" />
              )}
              Repair
            </Button>
          </TooltipTrigger>
          <TooltipContent>Run workspace consistency checks and recompile missing/invalid provider config as needed.</TooltipContent>
        </Tooltip>
      </div>

      <p className="text-[10px] text-muted-foreground">
        Integrity hash: <code>{detail.contextHash?.slice(0, 12) ?? 'n/a'}</code>
      </p>

      {lastRepairReport && (
        <div className="mt-3 rounded-lg border bg-muted/20 p-2">
          <p className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
            Repair Report
          </p>
          <p className="text-[11px] text-muted-foreground">
            needs_recompile={String(lastRepairReport.needs_recompile)} ·
            reapplied={String(lastRepairReport.reapplied_compile)} ·
            missing={lastRepairReport.missing_provider_configs.join(', ') || 'none'}
          </p>
        </div>
      )}
    </section>
  );
}
