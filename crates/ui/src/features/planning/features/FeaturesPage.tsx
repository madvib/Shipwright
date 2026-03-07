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

  const stats = useMemo(() => featureHubStats(features), [features]);

  const filteredFeatures = useMemo(() => {
    return features
      .filter((f) => {
        const matchesSearch =
          f.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
          f.file_name.toLowerCase().includes(searchQuery.toLowerCase());
        const matchesStatus =
          statusFilter.length === 0 || statusFilter.includes(f.status || 'draft');
        return matchesSearch && matchesStatus;
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
  }, [features, searchQuery, statusFilter, sortBy]);

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
            viewFilter="all"
            onViewFilterChange={() => { }}
            statusOptions={[]}
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
                const readiness = featureStatusFallbackReadiness(feature.status || 'draft');

                return (
                  <FeatureHubRow
                    key={feature.file_name}
                    feature={feature}
                    release={release}
                    spec={spec}
                    readiness={readiness}
                    isBlocking={feature.status !== 'implemented' && feature.status !== 'deprecated'}
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

      {selectedFeature && (
        <FeatureDetail
          feature={selectedFeature}
          onClose={onCloseFeatureDetail}
          onSave={onSaveFeature}
          onSelectRelease={() => {
            // Logic to handle navigation could go here or through parent
          }}
          onSelectSpec={() => {
            // Logic to handle navigation
          }}
          releaseSuggestions={releases.map(r => r.version)}
          specSuggestions={specs.map(s => s.file_name)}
          mcpEnabled={mcpEnabled}
        />
      )}
    </PageFrame>
  );
}
