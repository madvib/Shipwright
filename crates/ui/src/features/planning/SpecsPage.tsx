import { FormEvent, useEffect, useState } from 'react';
import { ArrowRight, FileCode2, Plus } from 'lucide-react';
import { SpecInfo as SpecEntry } from '@/bindings';
import DetailSheet from './DetailSheet';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import MarkdownEditor from '@/components/editor';

interface SpecsPageProps {
  specs: SpecEntry[];
  onSelectSpec: (entry: SpecEntry) => void;
  onCreateSpec: (title: string, content: string) => Promise<void>;
}

export default function SpecsPage({ specs, onSelectSpec, onCreateSpec }: SpecsPageProps) {
  const [createOpen, setCreateOpen] = useState(false);
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
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
      await onCreateSpec(cleanTitle, content);
      setCreateOpen(false);
      setTitle('');
      setContent('');
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
        const form = document.getElementById('new-spec-form') as HTMLFormElement | null;
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
          <h2 className="text-2xl font-semibold tracking-tight">Specs</h2>
          <p className="text-muted-foreground text-sm">
            Refine intent, then turn specs into actionable issues.
          </p>
        </div>
        <Button className="gap-2" onClick={() => setCreateOpen(true)}>
          <Plus className="size-4" />
          New Spec
        </Button>
      </header>

      {specs.length === 0 ? (
        <Card size="sm">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <FileCode2 className="size-4" />
              No specs yet
            </CardTitle>
            <CardDescription>Specs define scope, decisions, and delivery criteria.</CardDescription>
          </CardHeader>
          <CardContent>
            <Button onClick={() => setCreateOpen(true)}>
              <Plus className="size-4" />
              Create First Spec
            </Button>
          </CardContent>
        </Card>
      ) : (
        <Card size="sm">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Specification Library</CardTitle>
            <CardDescription>
              {specs.length} spec{specs.length !== 1 ? 's' : ''} in this project
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            {specs.map((spec) => (
              <div
                key={spec.path}
                className="hover:bg-muted/40 grid gap-2 rounded-md border p-3 transition-colors md:grid-cols-[1fr_auto] md:items-center"
                title={spec.path}
              >
                <div className="min-w-0">
                  <p className="truncate text-sm font-medium">{spec.title}</p>
                  <p className="text-muted-foreground truncate text-xs">{spec.file_name}</p>
                </div>
                <Button variant="outline" size="sm" onClick={() => onSelectSpec(spec)}>
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
          label="New Spec"
          title={<h2 className="text-xl font-semibold tracking-tight">Create Spec</h2>}
          meta={
            <p className="text-muted-foreground text-xs">
              Start with a title and optional markdown body.
            </p>
          }
          onClose={() => {
            if (creating) return;
            setCreateOpen(false);
          }}
          className="max-w-[1400px]"
          footer={
            <div className="flex flex-wrap items-center justify-end gap-2">
              <Button
                type="button"
                variant="outline"
                onClick={() => setCreateOpen(false)}
                disabled={creating}
              >
                Cancel
              </Button>
              <Button type="submit" form="new-spec-form" disabled={creating}>
                {creating ? 'Creating…' : 'Create Spec'}
              </Button>
            </div>
          }
        >
          <form id="new-spec-form" onSubmit={submitCreate} className="space-y-4">
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
                placeholder="Alpha Spec"
                onChange={(event) => {
                  setTitle(event.target.value);
                  setError(null);
                }}
                disabled={creating}
              />
            </div>
            <MarkdownEditor
              label="Content"
              value={content}
              onChange={setContent}
              placeholder="# Alpha Spec"
              rows={22}
              defaultMode="doc"
            />
          </form>
        </DetailSheet>
      )}
    </div>
  );
}
