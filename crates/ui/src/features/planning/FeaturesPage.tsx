import { FormEvent, useEffect, useMemo, useState } from 'react';
import { Flag, Plus } from 'lucide-react';
import {
  AdrEntry,
  FeatureDocument,
  FeatureInfo as FeatureEntry,
  ReleaseInfo as ReleaseEntry,
} from '@/bindings';
import { SpecInfo as SpecEntry } from '@/lib/types/spec';
import DetailSheet from './DetailSheet';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  Card,
  CardContent,
  CardHeader,
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import TemplateEditorButton from './TemplateEditorButton';
import MarkdownEditor from '@/components/editor';
import FeatureMetadataPanel from '@/components/editor/FeatureMetadataPanel';
import { EmptyState } from '@/components/ui/empty-state';
import { PageFrame, PageHeader } from '@/components/app/PageFrame';
import {
  readFrontmatterStringField,
  splitFrontmatterDocument,
  FrontmatterDelimiter,
} from '@/components/editor/frontmatter';
import {
  featureStatusFallbackReadiness,
  formatStatusLabel,
} from '@/features/planning/hub/utils/featureMetrics';
import FeatureHubStats from '@/features/planning/features-hub/components/FeatureHubStats';
import FeatureHubToolbar from '@/features/planning/features-hub/components/FeatureHubToolbar';
import FeatureHubRow from '@/features/planning/features-hub/components/FeatureHubRow';
import { useFeatureChecklistMetrics } from '@/features/planning/hub/hooks/useFeatureChecklistMetrics';
import FeatureDetail from './FeatureDetail';
import HubSectionHeader from '@/features/planning/hub/components/HubSectionHeader';

interface FeaturesPageProps {
  features: FeatureEntry[];
  releases: ReleaseEntry[];
  specs: SpecEntry[];
  adrs: AdrEntry[];
  selectedFeature: FeatureDocument | null;
  onCloseFeatureDetail: () => void;
  onSelectFeature: (entry: FeatureEntry) => void;
  onSelectReleaseFromFeature: (fileName: string) => void;
  onSelectSpecFromFeature: (fileName: string) => void;
  onSaveFeature: (fileName: string, content: string) => Promise<void> | void;
  onCreateFeature: (
    title: string,
    content: string,
    release?: string | null,
    spec?: string | null
  ) => Promise<void>;
  tagSuggestions?: string[];
  mcpEnabled?: boolean;
}

type FeatureSort = 'newest' | 'oldest' | 'status' | 'readiness';
const FEATURE_SORT_OPTIONS: Array<{ value: FeatureSort; label: string }> = [
  { value: 'newest', label: 'Newest first' },
  { value: 'oldest', label: 'Oldest first' },
  { value: 'status', label: 'Status' },
  { value: 'readiness', label: 'Readiness' },
];

type FeatureView = 'all' | 'blocking' | 'ready';

const STATUS_ORDER: Record<string, number> = {
  planned: 0,
  'in-progress': 1,
  implemented: 2,
  deprecated: 3,
};

export default function FeaturesPage({
  features,
  releases,
  specs,
  adrs,
  selectedFeature,
  onCloseFeatureDetail,
  onSelectFeature,
  onSelectReleaseFromFeature,
  onSelectSpecFromFeature,
  onSaveFeature,
  onCreateFeature,
  tagSuggestions = [],
  mcpEnabled = true,
}: FeaturesPageProps) {
  const updatedAt = (value?: string) => {
    const parsed = Date.parse(value ?? '');
    return Number.isFinite(parsed) ? parsed : 0;
  };

  const [createOpen, setCreateOpen] = useState(false);
  const [content, setContent] = useState('');
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sortBy, setSortBy] = useState<FeatureSort>('newest');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedStatuses, setSelectedStatuses] = useState<Set<string>>(new Set());
  const [viewFilter, setViewFilter] = useState<FeatureView>('all');
  const { metricsByFile: featureMetricsByFile } = useFeatureChecklistMetrics(features);

  const statusOptions = useMemo(() => {
    const fallback = ['planned', 'in-progress', 'implemented', 'deprecated'];
    const available = Array.from(
      new Set([
        ...fallback,
        ...features
          .map((feature) => feature.status)
          .filter((status): status is string => typeof status === 'string' && status.length > 0),
      ])
    );
    available.sort((a, b) => {
      const rankA = STATUS_ORDER[a] ?? 99;
      const rankB = STATUS_ORDER[b] ?? 99;
      if (rankA !== rankB) return rankA - rankB;
      return a.localeCompare(b);
    });

    return available.map((value) => ({
      value,
      label: formatStatusLabel(value),
    }));
  }, [features]);

  const sortedFeatures = useMemo(() => {
    const needle = searchQuery.trim().toLowerCase();
    const filtered = features.filter((feature) => {
      const featureStatus = feature.status ?? 'planned';
      const metric = featureMetricsByFile[feature.file_name];
      const readiness = metric?.readinessPercent ?? featureStatusFallbackReadiness(featureStatus);
      const blocking =
        metric?.blocking ?? (featureStatus !== 'implemented' && featureStatus !== 'deprecated');
      const ready = !blocking && readiness >= 90;

      const title = (feature.title ?? '').toLowerCase();
      const fileName = (feature.file_name ?? '').toLowerCase();
      const releaseId = (feature.release_id ?? '').toLowerCase();
      const specId = (feature.spec_id ?? '').toLowerCase();

      const matchesSearch =
        title.includes(needle) ||
        fileName.includes(needle) ||
        releaseId.includes(needle) ||
        specId.includes(needle);
      const matchesStatus = selectedStatuses.size === 0 || selectedStatuses.has(featureStatus);
      const matchesView =
        viewFilter === 'all' ||
        (viewFilter === 'blocking' ? blocking : ready);

      return matchesSearch && matchesStatus && matchesView;
    });

    filtered.sort((a, b) => {
      switch (sortBy) {
        case 'oldest':
          return updatedAt(a.updated) - updatedAt(b.updated);
        case 'status':
          return (STATUS_ORDER[a.status ?? 'planned'] ?? 99) - (STATUS_ORDER[b.status ?? 'planned'] ?? 99);
        case 'readiness': {
          const readinessA =
            featureMetricsByFile[a.file_name]?.readinessPercent ??
            featureStatusFallbackReadiness(a.status ?? 'planned');
          const readinessB =
            featureMetricsByFile[b.file_name]?.readinessPercent ??
            featureStatusFallbackReadiness(b.status ?? 'planned');
          return readinessB - readinessA;
        }
        case 'newest':
        default:
          return updatedAt(b.updated) - updatedAt(a.updated);
      }
    });

    return filtered;
  }, [featureMetricsByFile, features, searchQuery, selectedStatuses, sortBy, viewFilter]);

  const metrics = useMemo(() => {
    if (features.length === 0) {
      return {
        total: 0,
        implemented: 0,
        blocking: 0,
        unlinked: 0,
        avgReadiness: 0,
      };
    }

    let implemented = 0;
    let blocking = 0;
    let unlinked = 0;
    let readinessTotal = 0;

    for (const feature of features) {
      const featureStatus = feature.status ?? 'planned';
      if (featureStatus === 'implemented') {
        implemented += 1;
      }
      if (!feature.release_id) {
        unlinked += 1;
      }

      const metric = featureMetricsByFile[feature.file_name];
      const readiness = metric?.readinessPercent ?? featureStatusFallbackReadiness(featureStatus);
      readinessTotal += readiness;

      const isBlocking =
        metric?.blocking ?? (featureStatus !== 'implemented' && featureStatus !== 'deprecated');
      if (isBlocking) {
        blocking += 1;
      }
    }

    return {
      total: features.length,
      implemented,
      blocking,
      unlinked,
      avgReadiness: Math.round(readinessTotal / features.length),
    };
  }, [featureMetricsByFile, features]);

  const createInitialFeatureDocument = () => {
    return `+++
title = ""
status = "planned"
release_id = ""
spec_id = ""
adrs = []
tags = []
+++

## Why


## Acceptance Criteria

- [ ]

## Delivery Todos

- [ ]

## Notes
`;
  };

  const submitCreate = async (event: FormEvent) => {
    event.preventDefault();
    const parsed = splitFrontmatterDocument(content);
    const cleanTitle = readFrontmatterStringField(parsed.frontmatter, 'title').trim();
    if (!cleanTitle) {
      setError('Title is required.');
      return;
    }

    const release =
      readFrontmatterStringField(parsed.frontmatter, 'release_id').trim() ||
      readFrontmatterStringField(parsed.frontmatter, 'release').trim();
    const spec =
      readFrontmatterStringField(parsed.frontmatter, 'spec_id').trim() ||
      readFrontmatterStringField(parsed.frontmatter, 'spec').trim();

    try {
      setCreating(true);
      await onCreateFeature(
        cleanTitle,
        content,
        release ? release : null,
        spec ? spec : null
      );
      setCreateOpen(false);
      setContent(createInitialFeatureDocument());
      setError(null);
    } catch (createError) {
      setError(String(createError));
    } finally {
      setCreating(false);
    }
  };

  useEffect(() => {
    if (!createOpen) return;

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !creating) {
        event.preventDefault();
        setCreateOpen(false);
        return;
      }
      if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
        const form = document.getElementById('new-feature-form') as HTMLFormElement | null;
        form?.requestSubmit();
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [createOpen, creating]);

  useEffect(() => {
    if (createOpen) return;
    setContent(createInitialFeatureDocument());
  }, [createOpen]);

  if (selectedFeature) {
    return (
      <PageFrame width="wide">
        <FeatureDetail
          feature={selectedFeature}
          releaseSuggestions={releases.map((entry) => entry.file_name)}
          specSuggestions={specs.map((entry) => entry.file_name)}
          adrSuggestions={adrs.map((entry) => entry.file_name)}
          tagSuggestions={tagSuggestions}
          mcpEnabled={mcpEnabled}
          onClose={onCloseFeatureDetail}
          onSelectRelease={onSelectReleaseFromFeature}
          onSelectSpec={onSelectSpecFromFeature}
          onSave={onSaveFeature}
        />
      </PageFrame>
    );
  }

  return (
    <PageFrame width="wide">
      <PageHeader
        title="Features"
        actions={
          <div className="flex items-center gap-2">
            <TemplateEditorButton kind="feature" />
            <Button className="gap-2" onClick={() => setCreateOpen(true)}>
              <Plus className="size-4" />
              New Feature
            </Button>
          </div>
        }
      />

      <FeatureHubStats metrics={metrics} />

      {features.length === 0 ? (
        <EmptyState
          icon={<Flag className="size-4" />}
          title="No features yet"
          description="Create feature docs and track their delivery by release."
          action={
            <Button onClick={() => setCreateOpen(true)}>
              <Plus className="mr-2 size-4" />
              Create First Feature
            </Button>
          }
        />
      ) : (
        <Card size="sm">
          <CardHeader className="pb-2">
            <HubSectionHeader
              controls={
                <div className="flex flex-wrap items-center justify-end gap-2">
                  <FeatureHubToolbar
                    searchQuery={searchQuery}
                    onSearchQueryChange={setSearchQuery}
                    viewFilter={viewFilter}
                    onViewFilterChange={setViewFilter}
                    statusOptions={statusOptions}
                    selectedStatuses={selectedStatuses}
                    onSelectedStatusesChange={setSelectedStatuses}
                    sortBy={sortBy}
                    sortOptions={FEATURE_SORT_OPTIONS}
                    onSortByChange={(value) => setSortBy(value as FeatureSort)}
                  />
                </div>
              }
            />
          </CardHeader>
          <CardContent className="space-y-2">
            {sortedFeatures.length === 0 && (
              <div className="py-8 text-center text-sm text-muted-foreground italic">
                No features match the current search or filters.
              </div>
            )}

            {sortedFeatures.map((feature) => {
              const linkedRelease = releases.find((release) => release.file_name === feature.release_id);
              const linkedSpec = specs.find((spec) => spec.file_name === feature.spec_id);
              const metric = featureMetricsByFile[feature.file_name];
              const readiness = metric?.readinessPercent ?? featureStatusFallbackReadiness(feature.status);
              const isBlocking =
                metric?.blocking ?? (feature.status !== 'implemented' && feature.status !== 'deprecated');

              return (
                <FeatureHubRow
                  key={feature.path}
                  feature={feature}
                  release={linkedRelease ?? null}
                  spec={linkedSpec ?? null}
                  metrics={metric}
                  readiness={readiness}
                  isBlocking={isBlocking}
                  onSelect={onSelectFeature}
                />
              );
            })}
          </CardContent>
        </Card>
      )}

      {createOpen && (
        <DetailSheet
          label="New Feature"
          title={<h2 className="text-xl font-semibold tracking-tight">Create Feature</h2>}
          meta={
            <p className="text-muted-foreground text-xs">
              Add optional links to a release and a spec.
            </p>
          }
          onClose={() => {
            if (creating) return;
            setCreateOpen(false);
          }}
          className="max-w-[1400px]"
          footer={
            <div className="flex flex-wrap items-center justify-end gap-2">
              <Button type="button" variant="outline" onClick={() => setCreateOpen(false)} disabled={creating}>
                Cancel
              </Button>
              <Button type="submit" form="new-feature-form" disabled={creating}>
                {creating ? 'Creating…' : 'Create Feature'}
              </Button>
            </div>
          }
        >
          <form id="new-feature-form" onSubmit={submitCreate} className="space-y-4">
            {error && (
              <Alert variant="destructive">
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            <MarkdownEditor
              label="Feature Plan"
              value={content}
              onChange={(next: string) => {
                setContent(next);
                setError(null);
              }}
              frontmatterPanel={({ frontmatter, delimiter, onChange }: { frontmatter: string | null; delimiter: FrontmatterDelimiter | null; onChange: (fm: string | null, d: FrontmatterDelimiter) => void }) => (
                <FeatureMetadataPanel
                  frontmatter={frontmatter}
                  delimiter={delimiter}
                  defaultTitle=""
                  defaultStatus="planned"
                  releaseSuggestions={releases.map((entry: ReleaseEntry) => entry.file_name)}
                  specSuggestions={specs.map((entry: SpecEntry) => entry.file_name)}
                  adrSuggestions={adrs.map((entry: AdrEntry) => entry.file_name)}
                  tagSuggestions={tagSuggestions}
                  onChange={onChange}
                />
              )}
              placeholder="# Why this feature"
              rows={22}
              defaultMode="doc"
              mcpEnabled={mcpEnabled}
            />
          </form>
        </DetailSheet>
      )}
    </PageFrame>
  );
}
