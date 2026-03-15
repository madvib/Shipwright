import { FormEvent, useEffect, useMemo, useState } from 'react';
import { PackagePlus, Plus } from 'lucide-react';
import {
  FeatureInfo as FeatureEntry,
  ReleaseInfo,
  ReleaseDocument,
} from '@/bindings';
import { Alert, AlertDescription, Button, Tooltip, TooltipTrigger, TooltipContent, Card, CardContent, CardHeader, EmptyState, PageFrame, PageHeader, DetailSheet } from '@ship/primitives';
import MarkdownEditor from '@/components/editor';
import TemplateEditorButton from '../common/TemplateEditorButton';
import {
  readFrontmatterBooleanField,
  readFrontmatterStringField,
  readFrontmatterStringListField,
  splitFrontmatterDocument,
  setFrontmatterBooleanField,
  setFrontmatterStringField,
  setFrontmatterStringListField,
} from '@ship/primitives';
import { ReleaseHeaderMetadata } from './ReleaseHeaderMetadata';
import {
  featureStatusFallbackReadiness,
  FeatureChecklistMetrics,
} from '@/features/planning/common/hub/utils/featureMetrics';
import ReleaseHubStats from '@/features/planning/releases/hub/components/ReleaseHubStats';
import ReleaseHubToolbar from '@/features/planning/releases/hub/components/ReleaseHubToolbar';
import ReleaseHubRow from '@/features/planning/releases/hub/components/ReleaseHubRow';
import { useFeatureChecklistMetrics } from '@/features/planning/common/hub/hooks/useFeatureChecklistMetrics';
import ReleaseDetail from './ReleaseDetail';
import HubSectionHeader from '@/features/planning/common/hub/components/HubSectionHeader';

interface ReleasesPageProps {
  releases: ReleaseInfo[];
  features: FeatureEntry[];
  selectedRelease: ReleaseDocument | null;
  onCloseReleaseDetail: () => void;
  onSelectRelease: (entry: ReleaseInfo) => void;
  onSelectFeatureFromRelease: (feature: FeatureEntry) => void;
  onSaveRelease: (
    fileName: string,
    content: string,
    metadata?: {
      version?: string | null;
      status?: string | null;
      supported?: boolean | null;
      targetDate?: string | null;
      tags?: string[] | null;
    }
  ) => Promise<void> | void;
  onCreateRelease: (
    version: string,
    content: string,
    metadata?: {
      status?: string | null;
      supported?: boolean | null;
      targetDate?: string | null;
      tags?: string[] | null;
    }
  ) => Promise<void>;
  mcpEnabled?: boolean;
}

type ReleaseSort = 'newest' | 'oldest' | 'status' | 'progress';
const RELEASE_SORT_OPTIONS: Array<{ value: ReleaseSort; label: string }> = [
  { value: 'newest', label: 'Newest first' },
  { value: 'oldest', label: 'Oldest first' },
  { value: 'status', label: 'Status' },
  { value: 'progress', label: 'Progress' },
];

type ReleaseView = 'all' | 'blocking' | 'ready';

const RELEASE_STATUS_ORDER: Record<string, number> = {
  planned: 0,
  active: 1,
  shipped: 2,
  archived: 3,
};

interface ReleaseFeatureReadiness {
  feature: FeatureEntry;
  metrics?: FeatureChecklistMetrics;
  readiness: number;
  blocking: boolean;
  isActiveTarget: boolean;
}

interface ReleaseReadinessSummary {
  release: ReleaseInfo;
  scope: ReleaseFeatureReadiness[];
  linked: ReleaseFeatureReadiness[];
  activeTargets: ReleaseFeatureReadiness[];
  progressPercent: number;
  blockers: number;
  todosDone: number;
  todosTotal: number;
  acceptanceDone: number;
  acceptanceTotal: number;
  ready: boolean;
}

function parseVersionParts(rawVersion: string): {
  major: number;
  minor: number;
  patch: number;
  suffix: string | null;
} | null {
  const match = rawVersion.trim().match(/^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?$/);
  if (!match) return null;
  return {
    major: Number(match[1]),
    minor: Number(match[2]),
    patch: Number(match[3]),
    suffix: match[4] ?? null,
  };
}

function deriveNextReleaseVersion(releases: ReleaseInfo[]): string {
  const parsed = releases
    .map((release) => parseVersionParts(release.version ?? ''))
    .filter((value): value is NonNullable<ReturnType<typeof parseVersionParts>> => value !== null);

  if (parsed.length === 0) return 'v0.1.1-alpha';

  parsed.sort((left, right) => {
    if (left.major !== right.major) return left.major - right.major;
    if (left.minor !== right.minor) return left.minor - right.minor;
    return left.patch - right.patch;
  });
  const latest = parsed[parsed.length - 1];
  const nextPatch = latest.patch + 1;
  const suffix = latest.suffix ?? 'alpha';
  return `v${latest.major}.${latest.minor}.${nextPatch}-${suffix}`;
}

export default function ReleasesPage({
  releases,
  features,
  selectedRelease,
  onCloseReleaseDetail,
  onSelectRelease,
  onSelectFeatureFromRelease,
  onSaveRelease,
  onCreateRelease,
  mcpEnabled = true,
}: ReleasesPageProps) {
  const updatedAt = (value?: string) => {
    const parsed = Date.parse(value ?? '');
    return Number.isFinite(parsed) ? parsed : 0;
  };

  const [createOpen, setCreateOpen] = useState(false);
  const [content, setContent] = useState('');
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sortBy, setSortBy] = useState<ReleaseSort>('newest');
  const [search, setSearch] = useState('');
  const [viewFilter, setViewFilter] = useState<ReleaseView>('all');
  const [activeTargetsOnly, setActiveTargetsOnly] = useState(false);
  const { metricsByFile: featureMetricsByFile } = useFeatureChecklistMetrics(features);
  const defaultReleaseVersion = useMemo(() => deriveNextReleaseVersion(releases), [releases]);

  const matchesReleaseReference = (reference: string | null | undefined, release: ReleaseInfo) =>
    Boolean(reference) &&
    (reference === release.id || reference === release.file_name || reference === release.version);

  const createInitialReleaseDocument = () => {
    return `+++
  version = "${defaultReleaseVersion}"
status = "planned"
supported = false
target_date = ""
tags = []
+++

## Goal


## Scope

  - []

## Included Features

  - []

## Breaking Changes

  - [] None

## Notes
  `;
  };

  const documentModel = useMemo(() => splitFrontmatterDocument(content), [content]);
  const fm = documentModel.frontmatter;
  const currentVersion = readFrontmatterStringField(fm, 'version') || defaultReleaseVersion;
  const currentStatus = readFrontmatterStringField(fm, 'status') || 'planned';
  const currentSupported = readFrontmatterBooleanField(fm, 'supported') ?? false;
  const currentTargetDate = readFrontmatterStringField(fm, 'target_date');
  const currentTags = readFrontmatterStringListField(fm, 'tags');

  const handleMetadataUpdate = (updates: {
    version?: string;
    status?: string;
    supported?: boolean;
    target_date?: string;
    tags?: string[];
  }) => {
    let nextContent = content;
    const delimiter = documentModel.delimiter || '+++';

    if (updates.version !== undefined) nextContent = setFrontmatterStringField(nextContent, 'version', updates.version, delimiter) ?? nextContent;
    if (updates.status !== undefined) nextContent = setFrontmatterStringField(nextContent, 'status', updates.status, delimiter) ?? nextContent;
    if (updates.supported !== undefined) nextContent = setFrontmatterBooleanField(nextContent, 'supported', updates.supported, delimiter) ?? nextContent;
    if (updates.target_date !== undefined) nextContent = setFrontmatterStringField(nextContent, 'target_date', updates.target_date, delimiter) ?? nextContent;
    if (updates.tags !== undefined) nextContent = setFrontmatterStringListField(nextContent, 'tags', updates.tags, delimiter) ?? nextContent;

    if (nextContent !== content) {
      setContent(nextContent);
    }
  };

  const releaseSummaries = useMemo(() => {
    const summaries = new Map<string, ReleaseReadinessSummary>();
    for (const release of releases) {
      const linked = features
        .filter((feature) => matchesReleaseReference(feature.release_id, release))
        .map((feature) => {
          const metrics = featureMetricsByFile[feature.file_name];
          const readiness = metrics?.readinessPercent ?? featureStatusFallbackReadiness(feature.status);
          const blocking =
            metrics?.blocking ?? (feature.status !== 'implemented' && feature.status !== 'deprecated');
          return { feature, metrics, readiness, blocking, isActiveTarget: false };
        });

      const activeTargets = features
        .filter((feature) => {
          const effectiveActiveTarget = feature.active_target_id ?? feature.release_id;
          return matchesReleaseReference(effectiveActiveTarget, release);
        })
        .map((feature) => {
          const metrics = featureMetricsByFile[feature.file_name];
          const readiness = metrics?.readinessPercent ?? featureStatusFallbackReadiness(feature.status);
          const blocking =
            metrics?.blocking ?? (feature.status !== 'implemented' && feature.status !== 'deprecated');
          return { feature, metrics, readiness, blocking, isActiveTarget: true };
        });

      const scope = activeTargets.length > 0 ? activeTargets : linked;

      const blockers = scope.filter((entry) => entry.blocking).length;
      const todosDone = scope.reduce((sum, entry) => sum + (entry.metrics?.todos.done ?? 0), 0);
      const todosTotal = scope.reduce((sum, entry) => sum + (entry.metrics?.todos.total ?? 0), 0);
      const acceptanceDone = scope.reduce((sum, entry) => sum + (entry.metrics?.acceptance.done ?? 0), 0);
      const acceptanceTotal = scope.reduce((sum, entry) => sum + (entry.metrics?.acceptance.total ?? 0), 0);

      const statusWeightedProgress =
        scope.length > 0
          ? Math.round(
            scope.reduce((sum, entry) => sum + entry.readiness, 0) / scope.length
          )
          : 0;
      const progressPercent =
        todosTotal > 0 ? Math.round((todosDone / todosTotal) * 100) : statusWeightedProgress;
      const ready = scope.length > 0 && blockers === 0 && progressPercent >= 90;

      summaries.set(release.file_name, {
        release,
        scope,
        linked,
        activeTargets,
        progressPercent,
        blockers,
        todosDone,
        todosTotal,
        acceptanceDone,
        acceptanceTotal,
        ready,
      });
    }
    return summaries;
  }, [featureMetricsByFile, features, releases]);

  const dashboard = useMemo(() => {
    const active = releases.find((release) => release.status === 'active') ?? null;
    const shippedCount = releases.filter((release) => release.status === 'shipped').length;
    const activeBlockers = active
      ? releaseSummaries.get(active.file_name)?.blockers ?? 0
      : Array.from(releaseSummaries.values()).reduce((sum, summary) => sum + summary.blockers, 0);

    const progressValues = Array.from(releaseSummaries.values()).map((summary) => summary.progressPercent);
    const avgProgress =
      progressValues.length === 0
        ? 0
        : Math.round(progressValues.reduce((sum, value) => sum + value, 0) / progressValues.length);

    return {
      active,
      shippedCount,
      activeTargetFeatureCount: features.filter(
        (feature) => (feature.active_target_id ?? feature.release_id)
      ).length,
      activeTargetReleaseCount: Array.from(releaseSummaries.values()).filter(
        (summary) => summary.activeTargets.length > 0
      ).length,
      activeBlockers,
      avgProgress,
    };
  }, [features, releaseSummaries, releases]);

  const sortedReleases = useMemo(() => {
    const needle = search.trim().toLowerCase();
    const next = releases.filter((release) => {
      if (!needle) return true;
      const version = (release.version ?? '').toLowerCase();
      const status = (release.status ?? '').toLowerCase();
      const fileName = (release.file_name ?? '').toLowerCase();
      return (
        version.includes(needle) ||
        status.includes(needle) ||
        fileName.includes(needle)
      );
    });

    const viewFiltered = next.filter((release) => {
      const summary = releaseSummaries.get(release.file_name);
      if (!summary) return viewFilter === 'all';
      if (activeTargetsOnly && summary.activeTargets.length === 0) return false;
      if (viewFilter === 'blocking') return summary.blockers > 0;
      if (viewFilter === 'ready') return summary.ready;
      return true;
    });

    viewFiltered.sort((a, b) => {
      switch (sortBy) {
        case 'oldest':
          return updatedAt(a.updated) - updatedAt(b.updated);
        case 'status':
          return (RELEASE_STATUS_ORDER[a.status ?? 'planned'] ?? 99) - (RELEASE_STATUS_ORDER[b.status ?? 'planned'] ?? 99);
        case 'progress': {
          const progressA = releaseSummaries.get(a.file_name)?.progressPercent ?? 0;
          const progressB = releaseSummaries.get(b.file_name)?.progressPercent ?? 0;
          return progressB - progressA;
        }
        case 'newest':
        default:
          return updatedAt(b.updated) - updatedAt(a.updated);
      }
    });
    return viewFiltered;
  }, [activeTargetsOnly, releaseSummaries, releases, search, sortBy, viewFilter]);

  const submitCreate = async (event: FormEvent) => {
    event.preventDefault();
    const parsed = splitFrontmatterDocument(content);
    const cleanVersion = readFrontmatterStringField(parsed.frontmatter, 'version').trim();
    const status = readFrontmatterStringField(parsed.frontmatter, 'status').trim() || null;
    const targetDateValue = readFrontmatterStringField(parsed.frontmatter, 'target_date').trim();
    const targetDate = targetDateValue.length > 0 ? targetDateValue : null;
    const supported = readFrontmatterBooleanField(parsed.frontmatter, 'supported');
    const tags = readFrontmatterStringListField(parsed.frontmatter, 'tags');
    if (!cleanVersion) {
      setError('Version is required.');
      return;
    }
    try {
      setCreating(true);
      await onCreateRelease(cleanVersion, content, {
        status,
        supported,
        targetDate,
        tags,
      });
      setCreateOpen(false);
      setContent(createInitialReleaseDocument());
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
        const form = document.getElementById('new-release-form') as HTMLFormElement | null;
        form?.requestSubmit();
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [createOpen, creating]);

  useEffect(() => {
    if (createOpen) return;
    setContent(createInitialReleaseDocument());
  }, [createOpen, defaultReleaseVersion]);

  if (selectedRelease) {
    return (
      <PageFrame width="wide">
        <ReleaseDetail
          release={selectedRelease}
          features={features}
          onClose={onCloseReleaseDetail}
          onSelectFeature={onSelectFeatureFromRelease}
          onSave={onSaveRelease}
          mcpEnabled={mcpEnabled}
        />
      </PageFrame>
    );
  }

  return (
    <PageFrame width="wide">
      <PageHeader
        title="Releases"
        actions={
          <div className="flex items-center gap-2">
            <Tooltip>
              <TooltipTrigger asChild>
                <div>
                  <TemplateEditorButton kind="release" />
                </div>
              </TooltipTrigger>
              <TooltipContent side="bottom">Customize the template for release notes.</TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger asChild>
                <Button className="gap-2" onClick={() => setCreateOpen(true)}>
                  <Plus className="size-4" />
                  New Release
                </Button>
              </TooltipTrigger>
              <TooltipContent side="bottom">Start a new release milestone.</TooltipContent>
            </Tooltip>
          </div>
        }
      />

      <ReleaseHubStats
        activeRelease={dashboard.active}
        activeBlockers={dashboard.activeBlockers}
        shippedCount={dashboard.shippedCount}
        totalReleases={releases.length}
        activeTargetFeatureCount={dashboard.activeTargetFeatureCount}
        activeTargetReleaseCount={dashboard.activeTargetReleaseCount}
        avgProgress={dashboard.avgProgress}
      />

      {releases.length === 0 ? (
        <EmptyState
          icon={<PackagePlus className="size-4" />}
          title="No releases yet"
          description="Create an alpha milestone and attach features as you ship."
          action={
            <Button onClick={() => setCreateOpen(true)}>
              <Plus className="mr-2 size-4" />
              Create First Release
            </Button>
          }
        />
      ) : (
        <Card size="sm">
          <CardHeader className="pb-2">
            <HubSectionHeader
              controls={
                <ReleaseHubToolbar
                  search={search}
                  onSearchChange={setSearch}
                  viewFilter={viewFilter}
                  onViewFilterChange={setViewFilter}
                  activeTargetsOnly={activeTargetsOnly}
                  onActiveTargetsOnlyChange={setActiveTargetsOnly}
                  sortBy={sortBy}
                  sortOptions={RELEASE_SORT_OPTIONS}
                  onSortByChange={(value) => setSortBy(value as ReleaseSort)}
                />
              }
            />
          </CardHeader>
          <CardContent className="space-y-3">
            {sortedReleases.length === 0 && (
              <div className="py-8 text-center text-sm text-muted-foreground italic">
                No releases match the current search or filters.
              </div>
            )}

            {sortedReleases.map((release) => {
              const summary = releaseSummaries.get(release.file_name);
              const linked = summary?.scope ?? [];
              const progress = summary?.progressPercent ?? 0;
              const blockers = summary?.blockers ?? 0;

              return (
                <ReleaseHubRow
                  key={release.path}
                  release={release}
                  linked={linked}
                  activeTargetCount={summary?.activeTargets.length ?? 0}
                  linkedCount={summary?.linked.length ?? 0}
                  progress={progress}
                  blockers={blockers}
                  todosDone={summary?.todosDone ?? 0}
                  todosTotal={summary?.todosTotal ?? 0}
                  acceptanceDone={summary?.acceptanceDone ?? 0}
                  acceptanceTotal={summary?.acceptanceTotal ?? 0}
                  onOpen={onSelectRelease}
                />
              );
            })}
          </CardContent>
        </Card>
      )}

      {createOpen && (
        <DetailSheet
          label="New Release"
          title={<h2 className="text-xl font-semibold tracking-tight">Create Release</h2>}
          meta={
            <ReleaseHeaderMetadata
              version={currentVersion}
              status={currentStatus}
              supported={currentSupported}
              targetDate={currentTargetDate}
              tags={currentTags}
              isEditing={true}
              onUpdate={handleMetadataUpdate}
            />
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
              <Button type="submit" form="new-release-form" disabled={creating}>
                {creating ? 'Creating…' : 'Create Release'}
              </Button>
            </div>
          }
        >
          <form id="new-release-form" onSubmit={submitCreate} className="space-y-4">
            {error && (
              <Alert variant="destructive">
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            <MarkdownEditor
              label="Release Notes"
              value={content}
              onChange={(next) => {
                setContent(next);
                setError(null);
              }}
              placeholder="# Release Goal"
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
