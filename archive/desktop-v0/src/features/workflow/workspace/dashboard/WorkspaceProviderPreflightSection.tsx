import { Badge, Button, Tooltip, TooltipContent, TooltipTrigger } from '@ship/primitives';
import { Info, RefreshCw } from 'lucide-react';
import { ProviderInfo } from '@/bindings';
import { WorkspaceProviderMatrix } from '@/lib/platform/tauri/commands';

interface WorkspaceProviderPreflightSectionProps {
  providerMatrix: WorkspaceProviderMatrix | null;
  providerInfos: ProviderInfo[];
  loading: boolean;
  onRefresh: () => void;
}

function providerLabel(entry: ProviderInfo | undefined, id: string): string {
  const name = entry?.name?.trim();
  return name && name.length > 0 ? name : id;
}

export function WorkspaceProviderPreflightSection({
  providerMatrix,
  providerInfos,
  loading,
  onRefresh,
}: WorkspaceProviderPreflightSectionProps) {
  const byId = new Map(providerInfos.map((provider) => [provider.id, provider]));
  const allowed = providerMatrix?.allowed_providers ?? [];
  const supported = providerMatrix?.supported_providers ?? providerInfos.map((provider) => provider.id);
  const orderedIds = Array.from(new Set([...allowed, ...supported]));

  return (
    <section className="rounded-lg border bg-card p-3">
      <div className="mb-2 flex items-center justify-between gap-2">
        <div>
          <div className="flex items-center gap-1.5">
            <p className="text-[11px] font-semibold text-muted-foreground">Agent Readiness</p>
            <Tooltip>
              <TooltipTrigger asChild>
                <Info className="size-3 cursor-help text-muted-foreground/60" />
              </TooltipTrigger>
              <TooltipContent className="max-w-xs">
                Preflight checks provider install/connect/allow status before session start.
              </TooltipContent>
            </Tooltip>
          </div>
          <p className="text-[10px] text-muted-foreground">
            Source: {providerMatrix?.source ?? 'unknown'} · allowed: {allowed.length > 0 ? allowed.join(', ') : 'none'}
          </p>
        </div>
        <Button
          size="xs"
          variant="outline"
          className="h-7 gap-1 px-2 text-[11px]"
          onClick={onRefresh}
          disabled={loading}
        >
          {loading ? <RefreshCw className="size-3 animate-spin" /> : <RefreshCw className="size-3" />}
          Refresh
        </Button>
      </div>

      {providerMatrix?.resolution_error ? (
        <p className="mb-2 rounded-md border border-status-red/30 bg-status-red/5 px-2 py-1.5 text-[10px] text-status-red">
          {providerMatrix.resolution_error}
        </p>
      ) : null}

      <div className="space-y-2">
        {orderedIds.length === 0 ? (
          <p className="text-[11px] text-muted-foreground">
            No providers resolved. Run <code>ship providers detect</code> or connect one manually.
          </p>
        ) : (
          orderedIds.map((id) => {
            const info = byId.get(id);
            const isAllowed = allowed.includes(id);
            const isEnabled = info?.enabled ?? false;
            const isInstalled = info?.installed ?? false;
            const isReady = isAllowed && isEnabled && isInstalled;

            return (
              <div key={id} className="rounded-md border bg-muted/10 px-2 py-2">
                <div className="flex items-center justify-between gap-2">
                  <div className="min-w-0">
                    <p className="truncate text-xs font-medium">
                      {providerLabel(info, id)} <span className="text-muted-foreground">({id})</span>
                    </p>
                    <p className="text-[10px] text-muted-foreground">
                      binary: <code>{info?.binary ?? id}</code>
                      {info?.version ? ` · ${info.version}` : ''}
                    </p>
                  </div>
                  <div className="flex shrink-0 items-center gap-1">
                    <Badge variant={isReady ? 'default' : 'outline'} className="h-5 px-1.5 text-[9px]">
                      {isReady ? 'ready' : 'check'}
                    </Badge>
                    <Badge variant={isAllowed ? 'secondary' : 'outline'} className="h-5 px-1.5 text-[9px]">
                      {isAllowed ? 'allowed' : 'blocked'}
                    </Badge>
                    <Badge
                      variant={isEnabled ? 'secondary' : 'outline'}
                      className="h-5 px-1.5 text-[9px]"
                    >
                      {isEnabled ? 'connected' : 'disconnected'}
                    </Badge>
                    <Badge
                      variant={isInstalled ? 'secondary' : 'outline'}
                      className="h-5 px-1.5 text-[9px]"
                    >
                      {isInstalled ? 'installed' : 'missing'}
                    </Badge>
                  </div>
                </div>

                {!isInstalled ? (
                  <p className="mt-1 text-[10px] text-status-red">
                    Install <code>{info?.binary ?? id}</code> and ensure it is on PATH.
                  </p>
                ) : null}
                {!isEnabled ? (
                  <p className="mt-1 text-[10px] text-amber-700">
                    Run <code>ship providers connect {id}</code> to enable this provider.
                  </p>
                ) : null}
              </div>
            );
          })
        )}
      </div>
    </section>
  );
}
