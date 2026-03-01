import { FormEvent, useEffect, useMemo, useState } from 'react';
import { ArrowRight, Flag, Plus, Search } from 'lucide-react';
import { AdrEntry, FeatureInfo as FeatureEntry, ReleaseInfo as ReleaseEntry, SpecInfo as SpecEntry } from '@/bindings';
import DetailSheet from './DetailSheet';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import TemplateEditorButton from './TemplateEditorButton';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Input } from '@/components/ui/input';
import { StatusFilter } from '@/components/app/StatusFilter';
import MarkdownEditor from '@/components/editor';
import FeatureMetadataPanel from '@/components/editor/FeatureMetadataPanel';
import { EmptyState } from '@/components/ui/empty-state';
import { PageFrame, PageHeader } from '@/components/app/PageFrame';
import { readFrontmatterStringField, splitFrontmatterDocument } from '@/components/editor/frontmatter';
import { FrontmatterDelimiter } from '@/components/editor/frontmatter';

interface FeaturesPageProps {
  features: FeatureEntry[];
  releases: ReleaseEntry[];
  specs: SpecEntry[];
  adrs: AdrEntry[];
  onSelectFeature: (entry: FeatureEntry) => void;
  onCreateFeature: (
    title: string,
    content: string,
    release?: string | null,
    spec?: string | null
  ) => Promise<void>;
}

type FeatureSort = 'newest' | 'oldest' | 'status';
const FEATURE_SORT_OPTIONS: Array<{ value: FeatureSort; label: string }> = [
  { value: 'newest', label: 'Newest first' },
  { value: 'oldest', label: 'Oldest first' },
  { value: 'status', label: 'Status' },
];

export default function FeaturesPage({
  features,
  releases,
  specs,
  adrs,
  onSelectFeature,
  onCreateFeature,
}: FeaturesPageProps) {
  const [createOpen, setCreateOpen] = useState(false);
  const [content, setContent] = useState('');
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sortBy, setSortBy] = useState<FeatureSort>('newest');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedStatuses, setSelectedStatuses] = useState<Set<string>>(new Set());

  const sortedFeatures = useMemo(() => {
    const needle = searchQuery.trim().toLowerCase();
    const filtered = features.filter((feature) => {
      const matchesSearch =
        feature.title.toLowerCase().includes(needle) ||
        feature.file_name.toLowerCase().includes(needle);

      const matchesStatus = selectedStatuses.size === 0 || selectedStatuses.has(feature.status);

      return matchesSearch && matchesStatus;
    });

    filtered.sort((a, b) => {
      switch (sortBy) {
        case 'oldest':
          return new Date(a.updated).getTime() - new Date(b.updated).getTime();
        case 'status':
          return a.status.localeCompare(b.status, undefined, { sensitivity: 'base' });
        case 'newest':
        default:
          return new Date(b.updated).getTime() - new Date(a.updated).getTime();
      }
    });
    return filtered;
  }, [features, searchQuery, selectedStatuses, sortBy]);

  const createInitialFeatureDocument = () => {
    return `+++
title = ""
status = "active"
release_id = ""
spec = ""
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
    const release = readFrontmatterStringField(parsed.frontmatter, 'release_id').trim();
    const spec = readFrontmatterStringField(parsed.frontmatter, 'spec').trim();
    try {
      setCreating(true);
      await onCreateFeature(
        cleanTitle,
        content,
        release.trim() ? release.trim() : null,
        spec.trim() ? spec.trim() : null
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

  return (
    <PageFrame>
      <PageHeader
        title="Features"
        description="Plan customer-visible slices and bind them to releases/specs."
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
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div>
                <CardTitle className="text-sm">Feature Inventory</CardTitle>
                <CardDescription>
                  {features.length} feature{features.length !== 1 ? 's' : ''} in this project
                </CardDescription>
              </div>
              <div className="flex flex-1 flex-wrap items-center justify-end gap-2">
                <div className="relative min-w-[180px] flex-1 max-w-[280px]">
                  <Search className="text-muted-foreground absolute top-1/2 left-3 size-4 -translate-y-1/2" />
                  <Input
                    placeholder="Search features..."
                    className="pl-9 h-8"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                  />
                </div>
                <StatusFilter
                  label="Status"
                  options={[
                    { value: 'active', label: 'Active' },
                    { value: 'paused', label: 'Paused' },
                    { value: 'complete', label: 'Complete' },
                    { value: 'archived', label: 'Archived' },
                  ]}
                  selectedValues={selectedStatuses}
                  onSelect={setSelectedStatuses}
                />
                <Select value={sortBy} onValueChange={(value) => setSortBy(value as FeatureSort)}>
                  <SelectTrigger size="sm" className="w-[150px]">
                    <SelectValue>
                      {FEATURE_SORT_OPTIONS.find((option) => option.value === sortBy)?.label}
                    </SelectValue>
                  </SelectTrigger>
                  <SelectContent>
                    {FEATURE_SORT_OPTIONS.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-1.5">
            {sortedFeatures.length === 0 && (
              <div className="py-8 text-center text-sm text-muted-foreground italic">No features match the current filter.</div>
            )}
            {sortedFeatures.map((feature) => (
              <button
                key={feature.path}
                type="button"
                className="hover:bg-muted/40 grid w-full gap-2 rounded-md border p-3 text-left transition-colors md:grid-cols-[1fr_auto] md:items-center"
                title={feature.path}
                onClick={() => onSelectFeature(feature)}
              >
                <div className="min-w-0 space-y-1">
                  <p className="truncate text-sm font-medium">{feature.title}</p>
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge variant="outline">{feature.status}</Badge>
                    {feature.release_id && <Badge variant="secondary">{feature.release_id}</Badge>}
                  </div>
                </div>
                <ArrowRight className="size-4 text-muted-foreground/50 hidden md:block" />
              </button>
            ))}
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
                  defaultStatus="active"
                  releaseSuggestions={releases.map((entry: ReleaseEntry) => entry.file_name)}
                  specSuggestions={specs.map((entry: SpecEntry) => entry.file_name)}
                  adrSuggestions={adrs.map((entry: AdrEntry) => entry.file_name)}
                  tagSuggestions={[]}
                  onChange={onChange}
                />
              )}
              placeholder="# Why this feature"
              rows={22}
              defaultMode="doc"
            />
          </form>
        </DetailSheet>
      )}
    </PageFrame>
  );
}
