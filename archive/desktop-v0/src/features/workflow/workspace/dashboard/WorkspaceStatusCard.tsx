import { useMemo, useState } from 'react';
import {
  Badge,
  Button,
  Input,
  Popover,
  PopoverContent,
  PopoverTrigger,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@ship/primitives';
import {
  Archive,
  ExternalLink,
  Link2,
  RefreshCw,
  Sparkles,
  Wrench,
} from 'lucide-react';
import { type WorkspaceProviderMatrix, type WorkspaceRepairReport } from '@/lib/platform/tauri/commands';
import { WorkspaceGraphStatus } from '../../components/WorkspaceLifecycleGraph';
import { WorkspaceRow } from '../types';

interface WorkspaceStatusCardProps {
  detail: WorkspaceRow;
  statusVariant: (status: WorkspaceGraphStatus) => 'default' | 'secondary' | 'outline';
  linkedFeature: any;
  linkedRelease: any;
  linkFeatureId: string | null;
  setLinkFeatureId: (id: string | null) => void;
  linkReleaseId: string | null;
  setLinkReleaseId: (id: string | null) => void;
  featureLinkOptions: any[];
  releaseLinkOptions: any[];
  updatingLinks: boolean;
  onUpdateLinks: (featureId: string | null, releaseId: string | null) => void;
  onOpenFeature: () => void;
  onOpenRelease: () => void;
  onSync: () => void;
  syncing: boolean;
  onArchive: () => void;
  archiving: boolean;
  onRepair: () => void;
  repairing: boolean;
  lastRepairReport: WorkspaceRepairReport | null;
  providerMatrix: WorkspaceProviderMatrix | null;
}

export function WorkspaceStatusCard({
  detail,
  statusVariant,
  linkedFeature,
  linkedRelease,
  linkFeatureId,
  setLinkFeatureId,
  linkReleaseId,
  setLinkReleaseId,
  featureLinkOptions,
  releaseLinkOptions,
  updatingLinks,
  onUpdateLinks,
  onOpenFeature,
  onOpenRelease,
  onSync,
  syncing,
  onArchive,
  archiving,
  onRepair,
  repairing,
  lastRepairReport,
  providerMatrix,
}: WorkspaceStatusCardProps) {
  const [anchorSearch, setAnchorSearch] = useState('');

  const compileSummary = detail.compiledAt
    ? new Date(detail.compiledAt).toLocaleString()
    : 'not compiled yet';

  const search = anchorSearch.trim().toLowerCase();
  const filteredFeatures = useMemo(
    () =>
      featureLinkOptions.filter((feature) => {
        if (!search) return true;
        const id = String(feature.id ?? '').toLowerCase();
        const title = String(feature.title ?? '').toLowerCase();
        return id.includes(search) || title.includes(search);
      }),
    [featureLinkOptions, search],
  );

  const filteredReleases = useMemo(
    () =>
      releaseLinkOptions.filter((release) => {
        if (!search) return true;
        const id = String(release.id ?? '').toLowerCase();
        const version = String(release.version ?? '').toLowerCase();
        const fileName = String(release.file_name ?? '').toLowerCase();
        return id.includes(search) || version.includes(search) || fileName.includes(search);
      }),
    [releaseLinkOptions, search],
  );

  const linkButtonTitle = linkFeatureId
    ? 'Feature linked'
    : linkReleaseId
      ? 'Release linked'
      : 'No anchor linked';
  const effectiveProviders = (detail.providers ?? []).length > 0
    ? detail.providers
    : (providerMatrix?.allowed_providers ?? []);
  const providerSource = providerMatrix?.source ?? ((detail.providers ?? []).length > 0 ? 'workspace override' : 'default');

  return (
    <section className="rounded-xl border bg-card p-3 shadow-sm">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex min-w-0 flex-wrap items-center gap-1.5">
          <Badge variant={statusVariant(detail.status)} className="h-5 px-2 text-[9px] uppercase">
            {detail.status === 'active' ? 'open' : detail.status}
          </Badge>
          <Badge variant="outline" className="h-5 px-2 text-[9px] uppercase">
            {detail.workspaceType}
          </Badge>
          <Badge variant="outline" className="h-5 px-2 text-[9px] uppercase">
            {detail.isWorktree ? 'worktree' : 'checkout'}
          </Badge>
          <Tooltip>
            <TooltipTrigger asChild>
              <Badge variant="outline" className="h-5 max-w-[18rem] truncate px-2 text-[9px]">
                env: {detail.environmentId ?? 'default'}
              </Badge>
            </TooltipTrigger>
            <TooltipContent>{detail.environmentId ?? 'Workspace default environment'}</TooltipContent>
          </Tooltip>
          <Badge variant="outline" className="h-5 px-2 text-[9px]">
            providers: {(detail.providers ?? []).length}
          </Badge>
          <Tooltip>
            <TooltipTrigger asChild>
              <Badge variant="outline" className="h-5 px-2 text-[9px]">
                <Sparkles className="mr-1 size-3" />
                {detail.compileError ? 'compile error' : 'context compiled'}
              </Badge>
            </TooltipTrigger>
            <TooltipContent>Last compile: {compileSummary}</TooltipContent>
          </Tooltip>
          {detail.worktreePath ? (
            <Tooltip>
              <TooltipTrigger asChild>
                <Badge variant="outline" className="h-5 max-w-[20rem] truncate px-2 text-[9px]">
                  wt: {detail.worktreePath}
                </Badge>
              </TooltipTrigger>
              <TooltipContent className="max-w-xl break-all">{detail.worktreePath}</TooltipContent>
            </Tooltip>
          ) : null}

          {linkedFeature ? (
            <Tooltip>
              <TooltipTrigger asChild>
                <Badge variant="secondary" className="h-5 max-w-[18rem] gap-1 truncate px-2 text-[9px]">
                  feature: {linkedFeature.title || linkedFeature.id}
                  <button
                    type="button"
                    className="inline-flex items-center"
                    onClick={onOpenFeature}
                    aria-label="Open linked feature"
                  >
                    <ExternalLink className="size-3" />
                  </button>
                </Badge>
              </TooltipTrigger>
              <TooltipContent>Open linked feature</TooltipContent>
            </Tooltip>
          ) : null}

          {!linkedFeature && linkedRelease ? (
            <Tooltip>
              <TooltipTrigger asChild>
                <Badge variant="secondary" className="h-5 max-w-[18rem] gap-1 truncate px-2 text-[9px]">
                  release: {linkedRelease.version || linkedRelease.file_name || linkedRelease.id}
                  <button
                    type="button"
                    className="inline-flex items-center"
                    onClick={onOpenRelease}
                    aria-label="Open linked release"
                  >
                    <ExternalLink className="size-3" />
                  </button>
                </Badge>
              </TooltipTrigger>
              <TooltipContent>Open linked release</TooltipContent>
            </Tooltip>
          ) : null}
        </div>

        <div className="flex items-center gap-1">
          <Popover>
            <Tooltip>
              <TooltipTrigger asChild>
                <PopoverTrigger>
                  <Button size="icon-xs" variant="outline" className="size-7" disabled={updatingLinks}>
                    {updatingLinks ? <RefreshCw className="size-3.5 animate-spin" /> : <Link2 className="size-3.5" />}
                  </Button>
                </PopoverTrigger>
              </TooltipTrigger>
              <TooltipContent>{linkButtonTitle} · configure links</TooltipContent>
            </Tooltip>
            <PopoverContent className="w-[min(640px,94vw)] p-3" align="end" sideOffset={8}>
              <div className="space-y-3">
                <p className="text-[10px] text-muted-foreground">
                  Anchor this workspace to one feature or one release.
                </p>
                <Input
                  value={anchorSearch}
                  onChange={(event) => setAnchorSearch(event.target.value)}
                  placeholder="Search features and releases..."
                  className="h-8"
                />

                <div className="grid grid-cols-1 gap-2 md:grid-cols-2">
                  <div className="space-y-1 rounded-lg border bg-muted/20 p-2.5">
                    <div className="flex items-center justify-between gap-2">
                      <p className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">Feature</p>
                      {linkFeatureId ? (
                        <Button
                          size="xs"
                          variant="ghost"
                          className="h-6 px-1.5 text-[10px]"
                          onClick={() => {
                            setLinkFeatureId(null);
                            void onUpdateLinks(null, null);
                          }}
                        >
                          Clear
                        </Button>
                      ) : null}
                    </div>
                    <div className="max-h-40 space-y-1 overflow-y-auto">
                      {filteredFeatures.map((feature) => {
                        const title = feature.title || feature.id;
                        return (
                          <Button
                            key={feature.id}
                            size="xs"
                            variant={linkFeatureId === feature.id ? 'secondary' : 'ghost'}
                            className="h-8 w-full justify-start"
                            onClick={() => {
                              setLinkFeatureId(feature.id);
                              setLinkReleaseId(null);
                              void onUpdateLinks(feature.id, null);
                            }}
                            disabled={updatingLinks}
                            title={title}
                          >
                            <span className="truncate">{title}</span>
                          </Button>
                        );
                      })}
                      {filteredFeatures.length === 0 ? (
                        <p className="px-1 text-[10px] text-muted-foreground">No feature matches.</p>
                      ) : null}
                    </div>
                  </div>

                  <div className="space-y-1 rounded-lg border bg-muted/20 p-2.5">
                    <div className="flex items-center justify-between gap-2">
                      <p className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">Release</p>
                      {linkReleaseId ? (
                        <Button
                          size="xs"
                          variant="ghost"
                          className="h-6 px-1.5 text-[10px]"
                          onClick={() => {
                            setLinkReleaseId(null);
                            void onUpdateLinks(null, null);
                          }}
                        >
                          Clear
                        </Button>
                      ) : null}
                    </div>
                    <div className="max-h-40 space-y-1 overflow-y-auto">
                      {filteredReleases.map((release) => {
                        const title = release.version || release.file_name || release.id;
                        return (
                          <Button
                            key={release.id}
                            size="xs"
                            variant={linkReleaseId === release.id ? 'secondary' : 'ghost'}
                            className="h-8 w-full justify-start"
                            onClick={() => {
                              setLinkFeatureId(null);
                              setLinkReleaseId(release.id);
                              void onUpdateLinks(null, release.id);
                            }}
                            disabled={updatingLinks}
                            title={title}
                          >
                            <span className="truncate">{title}</span>
                          </Button>
                        );
                      })}
                      {filteredReleases.length === 0 ? (
                        <p className="px-1 text-[10px] text-muted-foreground">No release matches.</p>
                      ) : null}
                    </div>
                  </div>
                </div>
              </div>
            </PopoverContent>
          </Popover>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button size="icon-xs" variant="outline" className="size-7" onClick={onSync} disabled={syncing}>
                {syncing ? <RefreshCw className="size-3.5 animate-spin" /> : <RefreshCw className="size-3.5" />}
              </Button>
            </TooltipTrigger>
            <TooltipContent>Sync workspace record from git/worktree state</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button size="icon-xs" variant="outline" className="size-7" onClick={onRepair} disabled={repairing}>
                {repairing ? <RefreshCw className="size-3.5 animate-spin" /> : <Wrench className="size-3.5" />}
              </Button>
            </TooltipTrigger>
            <TooltipContent>Repair provider/compile drift</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                size="icon-xs"
                variant="outline"
                className="size-7"
                onClick={onArchive}
                disabled={archiving || detail.status === 'archived'}
              >
                {archiving ? <RefreshCw className="size-3.5 animate-spin" /> : <Archive className="size-3.5" />}
              </Button>
            </TooltipTrigger>
            <TooltipContent>Archive workspace</TooltipContent>
          </Tooltip>
        </div>
      </div>

      <div className="mt-2 rounded-md border bg-muted/15 px-2.5 py-2 text-[10px]">
        <p className="mb-1 font-semibold uppercase tracking-wide text-muted-foreground">Effective Agent Config</p>
        <div className="grid gap-2 md:grid-cols-3">
          <div className="space-y-1">
            <p className="text-muted-foreground">providers · {providerSource}</p>
            <div className="flex flex-wrap gap-1">
              {effectiveProviders.length > 0 ? (
                effectiveProviders.slice(0, 6).map((provider) => (
                  <Badge key={provider} variant="outline" className="h-4 px-1.5 text-[9px]">
                    {provider}
                  </Badge>
                ))
              ) : (
                <span className="text-muted-foreground">none</span>
              )}
              {effectiveProviders.length > 6 ? (
                <Badge variant="outline" className="h-4 px-1.5 text-[9px]">
                  +{effectiveProviders.length - 6}
                </Badge>
              ) : null}
            </div>
          </div>
          <div className="space-y-1">
            <p className="text-muted-foreground">mcp servers</p>
            <p className="font-mono text-[10px]">{detail.mcpServers.length > 0 ? detail.mcpServers.join(', ') : 'none'}</p>
          </div>
          <div className="space-y-1">
            <p className="text-muted-foreground">skills</p>
            <p className="font-mono text-[10px]">{detail.skills.length > 0 ? detail.skills.join(', ') : 'none'}</p>
          </div>
        </div>
        <p className="mt-2 text-muted-foreground">
          generation {detail.configGeneration}
          {detail.contextHash ? ` · context ${detail.contextHash.slice(0, 10)}…` : ''}
        </p>
      </div>

      {detail.compileError && (
        <div className="mt-2 rounded-md border border-status-red/30 bg-status-red/5 px-2 py-1 text-[10px] text-status-red">
          compile error: {detail.compileError}
        </div>
      )}

      {lastRepairReport && (
        <div className="mt-2 rounded-md border bg-muted/20 px-2 py-1 text-[10px] text-muted-foreground">
          repair: recompile={String(lastRepairReport.needs_recompile)} · reapplied={String(lastRepairReport.reapplied_compile)} · missing={lastRepairReport.missing_provider_configs.length}
        </div>
      )}
    </section>
  );
}
