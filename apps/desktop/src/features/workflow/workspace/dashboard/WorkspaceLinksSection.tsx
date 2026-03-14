import { useMemo } from 'react';
import {
  Button,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@ship/primitives';
import { ExternalLink, Info, RefreshCw } from 'lucide-react';

interface WorkspaceLinksSectionProps {
  linkedFeature: any;
  linkedSpec: any;
  linkFeatureId: string;
  setLinkFeatureId: (id: string) => void;
  linkSpecId: string;
  setLinkSpecId: (id: string) => void;
  featureLinkOptions: any[];
  specLinkOptions: any[];
  updatingLinks: boolean;
  onApplyLinks: () => void;
  onOpenFeature: () => void;
  onOpenSpec: () => void;
  noLinkValue: string;
}

export function WorkspaceLinksSection({
  linkedFeature,
  linkedSpec,
  linkFeatureId,
  setLinkFeatureId,
  linkSpecId,
  setLinkSpecId,
  featureLinkOptions,
  specLinkOptions,
  updatingLinks,
  onApplyLinks,
  onOpenFeature,
  onOpenSpec,
  noLinkValue,
}: WorkspaceLinksSectionProps) {
  const featureLabelById = useMemo(
    () =>
      new Map(
        featureLinkOptions.map((entry) => [entry.id, entry.title || entry.id])
      ),
    [featureLinkOptions]
  );

  const specLabelById = useMemo(
    () =>
      new Map(
        specLinkOptions.map((entry) => [
          entry.id,
          entry.spec?.metadata?.title || entry.id,
        ])
      ),
    [specLinkOptions]
  );

  const safeFeatureValue =
    linkFeatureId === noLinkValue || featureLabelById.has(linkFeatureId)
      ? linkFeatureId
      : noLinkValue;
  const safeSpecValue =
    linkSpecId === noLinkValue || specLabelById.has(linkSpecId)
      ? linkSpecId
      : noLinkValue;

  return (
    <section className="rounded-lg border bg-card p-3">
      <div className="mb-2 flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <p className="text-[11px] font-semibold text-muted-foreground">
            Linked artifacts
          </p>
          <Tooltip>
            <TooltipTrigger asChild>
              <Info className="size-3 cursor-help text-muted-foreground/50" />
            </TooltipTrigger>
            <TooltipContent side="top" className="max-w-xs">
              Link this workspace to a feature/spec so sessions roll up to the right
              project artifacts.
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
          {updatingLinks ? (
            <RefreshCw className="size-3 animate-spin" />
          ) : null}
          Apply
        </Button>
      </div>

      <div className="grid grid-cols-1 gap-2 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
        <div className="space-y-1">
          <div className="flex items-center justify-between gap-2">
            <span className="text-[10px] uppercase tracking-wide text-muted-foreground">
              Feature
            </span>
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
            onValueChange={(val) => setLinkFeatureId(val ?? noLinkValue)}
          >
            <SelectTrigger size="sm" className="h-8">
              <SelectValue placeholder="Unlinked">
                {(value) => {
                  if (!value || value === noLinkValue) return 'Unlinked';
                  return featureLabelById.get(String(value)) ?? 'Unlinked';
                }}
              </SelectValue>
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
            <span className="text-[10px] uppercase tracking-wide text-muted-foreground">
              Spec
            </span>
            {linkedSpec ? (
              <Button
                size="icon-xs"
                variant="ghost"
                className="size-5 text-muted-foreground"
                onClick={onOpenSpec}
              >
                <ExternalLink className="size-3" />
              </Button>
            ) : null}
          </div>
          <Select
            value={safeSpecValue}
            onValueChange={(val) => setLinkSpecId(val ?? noLinkValue)}
          >
            <SelectTrigger size="sm" className="h-8">
              <SelectValue placeholder="Unlinked">
                {(value) => {
                  if (!value || value === noLinkValue) return 'Unlinked';
                  return specLabelById.get(String(value)) ?? 'Unlinked';
                }}
              </SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={noLinkValue}>Unlinked</SelectItem>
              {specLinkOptions.map((entry) => (
                <SelectItem key={entry.id} value={entry.id}>
                  <span
                    className="block max-w-[24rem] truncate"
                    title={entry.spec?.metadata?.title || entry.id}
                  >
                    {entry.spec?.metadata?.title || entry.id}
                  </span>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
    </section>
  );
}
