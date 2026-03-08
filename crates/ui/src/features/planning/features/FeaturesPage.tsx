import { FormEvent, useMemo, useState } from 'react';
import { Flag, Plus } from 'lucide-react';
import {
  FeatureInfo as FeatureEntry,
  FeatureDocument,
  ReleaseInfo as ReleaseEntry,
  SpecEntry,
} from '@/bindings';
import {
  Alert,
  AlertDescription,
  Button,
  EmptyState,
  PageFrame,
  PageHeader,
  Tooltip,
  TooltipTrigger,
  TooltipContent,
  DetailSheet,
  MarkdownEditor,
} from '@ship/ui';
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
  specs: SpecEntry[];
  selectedFeature: FeatureDocument | null;
  onCloseFeatureDetail: () => void;
  onSelectFeature: (entry: FeatureEntry) => void;
  onSaveFeature: (fileName: string, content: string) => Promise<void> | void;
  onCreateFeature: (title: string, content: string) => Promise<void>;
  onSelectReleaseFromFeature?: (name: string) => void;
  onSelectSpecFromFeature?: (name: string) => void;
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
  specs,
  selectedFeature,
  onCloseFeatureDetail,
  onSelectFeature,
  onSaveFeature,
  onCreateFeature,
  onSelectReleaseFromFeature,
  onSelectSpecFromFeature,
  mcpEnabled = true,
}: FeaturesPageProps) {
  const [createOpen, setCreateOpen] = useState(false);
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<string[]>([]);
  const [sortBy, setSortBy] = useState<FeatureSort>('newest');
  const [viewFilter, setViewFilter] = useState<'all' | 'blocking' | 'ready'>('all');
  const { metricsByFile: featureMetricsByFile } = useFeatureChecklistMetrics(features);

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
      await onCreateFeature(title, content);
      setCreateOpen(false);
      setTitle('');
      setContent('');
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
                  <TooltipContent side="bottom">Create a new feature specification</TooltipContent>
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
                <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
                  {filteredFeatures.map((feature) => {
                    const release = releases.find(r => r.id === feature.release_id) ?? null;
                    const spec = specs.find(s => s.id === feature.spec_id) ?? null;
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
                        spec={spec}
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
          onSelectRelease={(name) => {
            if (onSelectReleaseFromFeature) {
              onSelectReleaseFromFeature(name);
            }
          }}
          onSelectSpec={(name) => {
            if (onSelectSpecFromFeature) {
              onSelectSpecFromFeature(name);
            }
          }}
          releaseSuggestions={releases.map(r => r.version)}
          specSuggestions={specs.map(s => s.file_name)}
          mcpEnabled={mcpEnabled}
        />
      )}

      {createOpen && (
        <DetailSheet
          title="New Feature"
          onClose={() => setCreateOpen(false)}
          footer={
            <div className="flex justify-end gap-2">
              <Button variant="ghost" onClick={() => setCreateOpen(false)}>
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
          </form>
        </DetailSheet>
      )}

    </PageFrame>
  );
}
