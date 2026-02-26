import { FormEvent, useEffect, useState } from 'react';
import { ArrowRight, Flag, Plus } from 'lucide-react';
import { FeatureInfo as FeatureEntry, ReleaseInfo as ReleaseEntry, SpecInfo as SpecEntry } from '@/bindings';
import DetailSheet from './DetailSheet';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import MarkdownEditor from '@/components/editor';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

interface FeaturesPageProps {
  features: FeatureEntry[];
  releases: ReleaseEntry[];
  specs: SpecEntry[];
  onSelectFeature: (entry: FeatureEntry) => void;
  onCreateFeature: (
    title: string,
    content: string,
    release?: string | null,
    spec?: string | null
  ) => Promise<void>;
}

const NONE_VALUE = '__none__';

export default function FeaturesPage({
  features,
  releases,
  specs,
  onSelectFeature,
  onCreateFeature,
}: FeaturesPageProps) {
  const [createOpen, setCreateOpen] = useState(false);
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [release, setRelease] = useState<string>(NONE_VALUE);
  const [spec, setSpec] = useState<string>(NONE_VALUE);
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const submitCreate = async (event: FormEvent) => {
    event.preventDefault();
    const cleanTitle = title.trim();
    if (!cleanTitle) {
      setError('Title is required.');
      return;
    }
    try {
      setCreating(true);
      await onCreateFeature(
        cleanTitle,
        content,
        release === NONE_VALUE ? null : release,
        spec === NONE_VALUE ? null : spec
      );
      setCreateOpen(false);
      setTitle('');
      setContent('');
      setRelease(NONE_VALUE);
      setSpec(NONE_VALUE);
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

  return (
    <div className="mx-auto flex w-full max-w-6xl flex-col gap-4 p-5 md:p-6">
      <header className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">Features</h2>
          <p className="text-muted-foreground text-sm">
            Plan customer-visible slices and bind them to releases/specs.
          </p>
        </div>
        <Button className="gap-2" onClick={() => setCreateOpen(true)}>
          <Plus className="size-4" />
          New Feature
        </Button>
      </header>

      {features.length === 0 ? (
        <Card size="sm">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Flag className="size-4" />
              No features yet
            </CardTitle>
            <CardDescription>Create feature docs and track their delivery by release.</CardDescription>
          </CardHeader>
          <CardContent>
            <Button onClick={() => setCreateOpen(true)}>
              <Plus className="size-4" />
              Create First Feature
            </Button>
          </CardContent>
        </Card>
      ) : (
        <Card size="sm">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Feature Inventory</CardTitle>
            <CardDescription>
              {features.length} feature{features.length !== 1 ? 's' : ''} in this project
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            {features.map((feature) => (
              <div
                key={feature.path}
                className="hover:bg-muted/40 grid gap-2 rounded-md border p-3 transition-colors md:grid-cols-[1fr_auto] md:items-center"
                title={feature.path}
              >
                <div className="min-w-0 space-y-1">
                  <p className="truncate text-sm font-medium">{feature.title}</p>
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge variant="outline">{feature.status}</Badge>
                    {feature.release && <Badge variant="secondary">{feature.release}</Badge>}
                  </div>
                </div>
                <Button variant="outline" size="sm" onClick={() => onSelectFeature(feature)}>
                  Open
                  <ArrowRight className="size-3.5" />
                </Button>
              </div>
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
              <div className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
                {error}
              </div>
            )}
            <div className="space-y-2">
              <label className="text-sm font-medium">Title</label>
              <Input
                autoFocus
                value={title}
                placeholder="Agent Mode Orchestrator"
                onChange={(event) => {
                  setTitle(event.target.value);
                  setError(null);
                }}
                disabled={creating}
              />
            </div>

            <div className="grid gap-3 md:grid-cols-2">
              <div className="space-y-2">
                <label className="text-sm font-medium">Release</label>
                <Select value={release} onValueChange={(value) => setRelease(value ?? NONE_VALUE)}>
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value={NONE_VALUE}>Unassigned</SelectItem>
                    {releases.map((entry) => (
                      <SelectItem key={entry.file_name} value={entry.file_name}>
                        {entry.version}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Spec</label>
                <Select value={spec} onValueChange={(value) => setSpec(value ?? NONE_VALUE)}>
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value={NONE_VALUE}>None</SelectItem>
                    {specs.map((entry) => (
                      <SelectItem key={entry.file_name} value={entry.file_name}>
                        {entry.title}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>

            <MarkdownEditor
              label="Feature Plan"
              value={content}
              onChange={setContent}
              placeholder="# Why this feature"
              rows={22}
              defaultMode="doc"
            />
          </form>
        </DetailSheet>
      )}
    </div>
  );
}
