import { FormEvent, useMemo, useState } from 'react';
import { Flag, Plus } from 'lucide-react';
import {
  FeatureInfo as FeatureEntry,
  FeatureDocument,
  ReleaseInfo as ReleaseEntry,
} from '@/bindings';
import {
  Alert,
  AlertDescription,
  Button,
  Input,
  EmptyState,
  PageFrame,
  PageHeader,
  Tooltip,
  TooltipTrigger,
  TooltipContent,
  DetailSheet,
  MarkdownEditor,
} from '@ship/primitives';
import {
  featureStatusFallbackReadiness,
} from '@/features/planning/common/hub/utils/featureMetrics';
import FeatureHubStats from '@/features/planning/features/hub/components/FeatureHubStats';
import FeatureHubToolbar from '@/features/planning/features/hub/components/FeatureHubToolbar';
import FeatureHubRow from '@/features/planning/features/hub/components/FeatureHubRow';
import { featureHubStats } from '@/features/planning/features/hub/utils/featureHubStats';
import FeatureDetail from './FeatureDetail';
import TemplateEditorButton from '../common/TemplateEditorButton';
import { useFeatureChecklistMetrics } from '@/features/planning/common/hub/hooks/useFeatureChecklistMetrics';
import { formatStatusLabel } from '@/features/planning/common/hub/utils/featureMetrics';


interface FeaturesPageProps {
  features: FeatureEntry[];
  releases: ReleaseEntry[];
  selectedFeature: FeatureDocument | null;
  onCloseFeatureDetail: () => void;
  onSelectFeature: (entry: FeatureEntry) => void;
  onSaveFeature: (fileName: string, content: string) => Promise<void> | void;
  onStartFeature: (fileName: string) => Promise<void> | void;
  onDoneFeature: (fileName: string) => Promise<void> | void;
  onSaveFeatureDocumentation: (
    fileName: string,
    content: string,
    status?: string | null,
    verifyNow?: boolean
  ) => Promise<void> | void;
  onCreateFeature: (
    title: string,
    content: string,
    releaseId?: string | null,
    branch?: string | null,
  ) => Promise<void>;
  onSelectReleaseFromFeature?: (name: string) => void;
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

export default function FeaturesPage({
  features,
  releases,
  selectedFeature,
  onCloseFeatureDetail,
  onSelectFeature,
  onSaveFeature,
  onStartFeature,
  onDoneFeature,
  onSaveFeatureDocumentation,
  onCreateFeature,
  onSelectReleaseFromFeature,
  mcpEnabled = true,
}: FeaturesPageProps) {
  const [createOpen, setCreateOpen] = useState(false);
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [createReleaseId, setCreateReleaseId] = useState<string>('');
  const [createBranch, setCreateBranch] = useState<string>('');
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<string[]>([]);
  const [sortBy, setSortBy] = useState<FeatureSort>('newest');
  const [viewFilter, setViewFilter] = useState<'all' | 'blocking' | 'ready'>('all');
  const { metricsByFile: featureMetricsByFile } = useFeatureChecklistMetrics(features);

  const resetCreateFeatureDraft = () => {
    setCreateOpen(false);
    setTitle('');
    setContent('');
    setCreateReleaseId('');
    setCreateReleaseId('');
    setCreateBranch('');
    setError(null);
  };

  const stats = useMemo(() => featureHubStats(features), [features]);

  const statusOptions = useMemo(() => {
    const canonical = ['planned', 'in-progress', 'implemented', 'deprecated'];
    const seen = features
      .map((feature) => feature.status)
      .filter((status): status is string => Boolean(status));
    const ordered = [...canonical, ...seen].filter(
      (status, index, arr) => arr.indexOf(status) === index
    );

    return ordered.map((status) => ({
      value: status,
      label: formatStatusLabel(status),
    }));
  }, [features]);

  const filteredFeatures = useMemo(() => {
    return features
      .filter((f) => {
        const matchesSearch =
          f.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
          f.file_name.toLowerCase().includes(searchQuery.toLowerCase());
        const matchesStatus =
          statusFilter.length === 0 || statusFilter.includes(f.status || 'draft');
        const metrics = featureMetricsByFile[f.file_name];
        const blocking =
          metrics?.blocking ?? (f.status !== 'implemented' && f.status !== 'deprecated');
        const readiness =
          metrics?.readinessPercent ?? featureStatusFallbackReadiness(f.status || 'draft');
        const matchesView =
          viewFilter === 'all' ||
          (viewFilter === 'blocking' && blocking) ||
          (viewFilter === 'ready' && !blocking && readiness >= 90);

        return matchesSearch && matchesStatus && matchesView;
      })
      .sort((a, b) => {
        if (sortBy === 'newest') {
          return (
            new Date(b.updated).getTime() - new Date(a.updated).getTime()
          );
        }
        if (sortBy === 'oldest') {
          return (
            new Date(a.updated).getTime() - new Date(b.updated).getTime()
          );
        }
        if (sortBy === 'status') {
          return (a.status || '').localeCompare(b.status || '');
        }
        if (sortBy === 'readiness') {
          const readinessA = featureStatusFallbackReadiness(a.status || 'draft');
          const readinessB = featureStatusFallbackReadiness(b.status || 'draft');
          return readinessB - readinessA;
        }
        return 0;
      });
  }, [featureMetricsByFile, features, searchQuery, sortBy, statusFilter, viewFilter]);

  const handleCreate = async (event: FormEvent) => {
    event.preventDefault();
    if (!title.trim()) return;

    try {
      setCreating(true);
      setError(null);
      await onCreateFeature(
        title,
        content,
        createReleaseId.trim() || null,
        createBranch.trim() || null,
      );
      resetCreateFeatureDraft();
    } catch (err) {
      setError(String(err));
    } finally {
      setCreating(false);
    }
  };

  return (
    <PageFrame width="wide">
      {!selectedFeature ? (
        <>
          <PageHeader
            title="Features Hub"
            description="Design and track functional capabilities"
            actions={
              <div className="flex items-center gap-2">
                <TemplateEditorButton kind="feature" />
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button onClick={() => setCreateOpen(true)} size="sm" className="h-8">
                      <Plus className="mr-1.5 size-3.5" />
                      New Feature
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="bottom">Create a new feature</TooltipContent>
                </Tooltip>
              </div>
            }
          />

          <div className="space-y-6">
            <FeatureHubStats metrics={stats} />

            <div className="space-y-4">
              <FeatureHubToolbar
                searchQuery={searchQuery}
                onSearchQueryChange={setSearchQuery}
                viewFilter={viewFilter}
                onViewFilterChange={setViewFilter}
                statusOptions={statusOptions}
                selectedStatuses={new Set(statusFilter)}
                onSelectedStatusesChange={(next) => setStatusFilter(Array.from(next))}
                sortBy={sortBy}
                sortOptions={FEATURE_SORT_OPTIONS}
                onSortByChange={(next) => setSortBy(next as FeatureSort)}
              />

              {filteredFeatures.length > 0 ? (
                <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">

                  {filteredFeatures.map((feature) => {
                    const release = releases.find(r => r.id === feature.release_id) ?? null;
                    const metrics = featureMetricsByFile[feature.file_name];
                    const readiness =
                      metrics?.readinessPercent ??
                      featureStatusFallbackReadiness(feature.status || 'draft');
                    const isBlocking =
                      metrics?.blocking ??
                      (feature.status !== 'implemented' && feature.status !== 'deprecated');

                    return (
                      <FeatureHubRow
                        key={feature.file_name}
                        feature={feature}
                        release={release}
                        readiness={readiness}
                        isBlocking={isBlocking}
                        onSelect={() => onSelectFeature(feature)}
                      />
                    );
                  })}
                </div>
              ) : (
                <EmptyState
                  icon={<Flag />}
                  title="No features found"
                  description="Refine your search or create a new feature to get started."
                  action={
                    <Button variant="outline" onClick={() => {
                      setSearchQuery('');
                      setStatusFilter([]);
                    }}>
                      Clear Filters
                    </Button>
                  }
                />
              )}
            </div>
          </div>
        </>
      ) : (
        <FeatureDetail
          feature={selectedFeature}
          onClose={onCloseFeatureDetail}
          onSave={onSaveFeature}
          onStart={onStartFeature}
          onDone={onDoneFeature}
          onSaveDocumentation={onSaveFeatureDocumentation}
          onSelectRelease={(name) => {
            if (onSelectReleaseFromFeature) {
              onSelectReleaseFromFeature(name);
            }
          }}
          releaseSuggestions={releases.map(r => r.version)}
          mcpEnabled={mcpEnabled}
        />
      )}

      {createOpen && (
        <DetailSheet
          title="New Feature"
          onClose={resetCreateFeatureDraft}
          footer={
            <div className="flex justify-end gap-2">
              <Button variant="ghost" onClick={resetCreateFeatureDraft}>
                Cancel
              </Button>
              <Button
                onClick={handleCreate}
                disabled={creating || !title.trim()}
              >
                {creating ? 'Creating...' : 'Create Feature'}
              </Button>
            </div>
          }
        >
          <form onSubmit={handleCreate} className="space-y-4">
            {error && (
              <Alert variant="destructive">
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            <div className="space-y-2">
              <label className="text-sm font-medium">Title</label>
              <input
                autoFocus
                className="bg-background w-full rounded-md border px-3 py-2"
                placeholder="Feature title..."
                value={title}
                onChange={(e) => setTitle(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">Initial Content (Optional)</label>
              <div className="h-[400px] overflow-hidden rounded-md border">
                <MarkdownEditor
                  value={content}
                  onChange={setContent}
                  defaultMode="doc"
                  fillHeight
                />
              </div>
            </div>
            <div className="grid gap-3 md:grid-cols-2">
              <div className="space-y-2">
                <label className="text-sm font-medium">Release Link (Optional)</label>
                <Input
                  list="feature-create-releases"
                  value={createReleaseId}
                  placeholder="release id (ex: rel_123...)"
                  onChange={(event) => setCreateReleaseId(event.target.value)}
                />
                <datalist id="feature-create-releases">
                  {releases.map((release) => (
                    <option key={release.id} value={release.id}>
                      {release.version}
                    </option>
                  ))}
                </datalist>
              </div>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">Branch (Optional)</label>
              <Input
                value={createBranch}
                placeholder="feature/your-scope"
                onChange={(event) => setCreateBranch(event.target.value)}
              />
              <p className="text-muted-foreground text-xs">
                Initialize the feature with a branch link so lifecycle checks and workspace handoff are ready.
              </p>
            </div>
          </form>
        </DetailSheet>
      )}

    </PageFrame>
  );
}
