import { useMemo } from 'react';
import {
  Badge,
  Button,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@ship/ui';
import { ExternalLink, Info, RefreshCw } from 'lucide-react';
import { type WorkspaceSessionInfo } from '@/lib/platform/tauri/commands';

interface WorkspaceLinksSectionProps {
  linkedFeature: any;
  linkedRelease: any;
  linkFeatureId: string;
  setLinkFeatureId: (id: string) => void;
  linkReleaseId: string;
  setLinkReleaseId: (id: string) => void;
  featureLinkOptions: any[];
  specLinkOptions: any[];
  releaseLinkOptions: any[];
  recentSessions: WorkspaceSessionInfo[];
  updatingLinks: boolean;
  onApplyLinks: () => void;
  onOpenFeature: () => void;
  onOpenRelease: () => void;
  noLinkValue: string;
}

export function WorkspaceLinksSection({
  linkedFeature,
  linkedRelease,
  linkFeatureId,
  setLinkFeatureId,
  linkReleaseId,
  setLinkReleaseId,
  featureLinkOptions,
  specLinkOptions,
  releaseLinkOptions,
  recentSessions,
  updatingLinks,
  onApplyLinks,
  onOpenFeature,
  onOpenRelease,
  noLinkValue,
}: WorkspaceLinksSectionProps) {
  const featureLabelById = useMemo(
    () => new Map(featureLinkOptions.map((entry) => [entry.id, entry.title || entry.id])),
    [featureLinkOptions],
  );

  const specLabelById = useMemo(
    () =>
      new Map(
        specLinkOptions.map((entry) => [entry.id, entry.spec?.metadata?.title || entry.id]),
      ),
    [specLinkOptions],
  );

  const releaseLabelById = useMemo(
    () =>
      new Map(
        releaseLinkOptions.map((entry) => [entry.id, entry.version || entry.file_name || entry.id]),
      ),
    [releaseLinkOptions],
  );

  const safeFeatureValue =
    linkFeatureId === noLinkValue || featureLabelById.has(linkFeatureId)
      ? linkFeatureId
      : noLinkValue;

  const resolvedReleaseOptionId = useMemo(() => {
    if (linkReleaseId === noLinkValue) return noLinkValue;
    const matched = releaseLinkOptions.find(
      (entry) =>
        entry.id === linkReleaseId ||
        entry.version === linkReleaseId ||
        entry.file_name === linkReleaseId,
    );
    return matched?.id ?? noLinkValue;
  }, [linkReleaseId, noLinkValue, releaseLinkOptions]);

  const safeReleaseValue =
    resolvedReleaseOptionId === noLinkValue || releaseLabelById.has(resolvedReleaseOptionId)
      ? resolvedReleaseOptionId
      : noLinkValue;

  const hasAnchorConflict =
    safeFeatureValue !== noLinkValue && safeReleaseValue !== noLinkValue;

  const touchedSpecIds = useMemo(() => {
    const ids = new Set<string>();
    for (const session of recentSessions) {
      for (const specId of session.updated_spec_ids ?? []) {
        if (specId?.trim()) ids.add(specId);
      }
    }
    return Array.from(ids);
  }, [recentSessions]);

  const visibleTouchedSpecs = touchedSpecIds
    .map((id) => ({ id, label: specLabelById.get(id) ?? id }))
    .slice(0, 8);

  return (
    <section className="rounded-lg border bg-card p-3">
      <div className="mb-2 flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <p className="text-[11px] font-semibold text-muted-foreground">Workspace Links</p>
          <Tooltip>
            <TooltipTrigger asChild>
              <Info className="size-3 cursor-help text-muted-foreground/50" />
            </TooltipTrigger>
            <TooltipContent side="top" className="max-w-xs">
              Workspace links store one planning anchor (feature OR release). Specs are attached per session.
            </TooltipContent>
          </Tooltip>
        </div>
        <Button
          size="xs"
          variant="outline"
          className="h-7 gap-1 px-2 text-[11px]"
          onClick={onApplyLinks}
          disabled={updatingLinks}
        >
          {updatingLinks ? <RefreshCw className="size-3 animate-spin" /> : null}
          Apply
        </Button>
      </div>

      <div className="space-y-2">
        <div className="grid grid-cols-1 gap-2 md:grid-cols-2">
          <div className="space-y-1">
            <div className="flex items-center justify-between gap-2">
              <span className="text-[10px] uppercase tracking-wide text-muted-foreground">Feature Anchor</span>
              {linkedFeature ? (
                <Button
                  size="icon-xs"
                  variant="ghost"
                  className="size-5 text-muted-foreground"
                  onClick={onOpenFeature}
                >
                  <ExternalLink className="size-3" />
                </Button>
              ) : null}
            </div>
            <Select
              value={safeFeatureValue}
              onValueChange={(val) => {
                const next = val ?? noLinkValue;
                setLinkFeatureId(next);
                if (next !== noLinkValue) {
                  setLinkReleaseId(noLinkValue);
                }
              }}
            >
              <SelectTrigger size="sm" className="h-8">
                <SelectValue placeholder="Unlinked" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={noLinkValue}>Unlinked</SelectItem>
                {featureLinkOptions.map((entry) => (
                  <SelectItem key={entry.id} value={entry.id}>
                    <span className="block max-w-[24rem] truncate" title={entry.title || entry.id}>
                      {entry.title || entry.id}
                    </span>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-1">
            <div className="flex items-center justify-between gap-2">
              <span className="text-[10px] uppercase tracking-wide text-muted-foreground">Release Anchor</span>
              {linkedRelease ? (
                <Button
                  size="icon-xs"
                  variant="ghost"
                  className="size-5 text-muted-foreground"
                  onClick={onOpenRelease}
                >
                  <ExternalLink className="size-3" />
                </Button>
              ) : null}
            </div>
            <Select
              value={safeReleaseValue}
              onValueChange={(val) => {
                const next = val ?? noLinkValue;
                setLinkReleaseId(next);
                if (next !== noLinkValue) {
                  setLinkFeatureId(noLinkValue);
                }
              }}
            >
              <SelectTrigger size="sm" className="h-8">
                <SelectValue placeholder="Unlinked" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={noLinkValue}>Unlinked</SelectItem>
                {releaseLinkOptions.map((entry) => (
                  <SelectItem key={entry.id} value={entry.id}>
                    <span
                      className="block max-w-[24rem] truncate"
                      title={entry.version || entry.file_name || entry.id}
                    >
                      {entry.version || entry.file_name || entry.id}
                    </span>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        {hasAnchorConflict && (
          <p className="rounded-md border border-amber-500/30 bg-amber-500/5 px-2 py-1 text-[10px] text-amber-700">
            Both feature and release are set. Choose one anchor before applying changes.
          </p>
        )}

        <div className="rounded-md border bg-muted/20 px-2.5 py-2">
          <p className="text-[10px] text-muted-foreground">Recent Session Specs</p>
          {visibleTouchedSpecs.length === 0 ? (
            <p className="mt-1 text-[10px] text-muted-foreground">No session-updated specs yet.</p>
          ) : (
            <div className="mt-1 flex flex-wrap gap-1">
              {visibleTouchedSpecs.map((spec) => (
                <Badge key={spec.id} variant="outline" className="text-[9px]">
                  {spec.label}
                </Badge>
              ))}
            </div>
          )}
        </div>
      </div>
    </section>
  );
}
