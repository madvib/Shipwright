import { FormEvent, useEffect, useMemo, useState } from 'react';
import { ArrowRight, FileCode2, Plus } from 'lucide-react';
import { SpecInfo as SpecEntry } from '@/lib/types/spec';
import {
  Alert,
  AlertDescription,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  EmptyState,
  DetailSheet,
  PageFrame,
  PageHeader,
  MarkdownEditor,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Input,
  Tooltip,
  TooltipTrigger,
  TooltipContent,
  readFrontmatterStringField,
  readFrontmatterStringListField,
  splitFrontmatterDocument,
  setFrontmatterStringListField,
} from '@ship/ui';
import { SpecHeaderMetadata } from './SpecHeaderMetadata';
import TemplateEditorButton from '../common/TemplateEditorButton';

interface SpecsPageProps {
  specs: SpecEntry[];
  tagSuggestions?: string[];
  onSelectSpec: (entry: SpecEntry) => void;
  onCreateSpec: (title: string, content: string) => Promise<void>;
}

type SpecSort = 'filename-asc' | 'filename-desc';
const SPEC_SORT_OPTIONS: Array<{ value: SpecSort; label: string }> = [
  { value: 'filename-asc', label: 'Filename A-Z' },
  { value: 'filename-desc', label: 'Filename Z-A' },
];

export default function SpecsPage({
  specs,
  tagSuggestions = [],
  onSelectSpec,
  onCreateSpec,
}: SpecsPageProps) {
  const [createOpen, setCreateOpen] = useState(false);
  const [content, setContent] = useState('');
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sortBy, setSortBy] = useState<SpecSort>('filename-asc');
  const [search, setSearch] = useState('');

  const sortedSpecs = useMemo(() => {
    const needle = search.trim().toLowerCase();
    const next = specs.filter((spec) => {
      if (!needle) return true;
      return (
        spec.spec.metadata.title.toLowerCase().includes(needle) ||
        spec.file_name.toLowerCase().includes(needle)
      );
    });
    next.sort((a, b) => {
      switch (sortBy) {
        case 'filename-asc':
          return a.file_name.localeCompare(b.file_name, undefined, { sensitivity: 'base' });
        case 'filename-desc':
          return b.file_name.localeCompare(a.file_name, undefined, { sensitivity: 'base' });
        default:
          return 0;
      }
    });
    return next;
  }, [search, sortBy, specs]);

  const createInitialSpecDocument = () => {
    return `+++
title = ""
status = "draft"
author = ""
tags = []
+++

## Overview


## Goals


## Non-Goals


## Approach


## Open Questions
`;
  };

  const documentModel = useMemo(() => splitFrontmatterDocument(content), [content]);
  const currentTags = useMemo(
    () => readFrontmatterStringListField(documentModel.frontmatter, 'tags'),
    [documentModel.frontmatter]
  );

  const handleMetadataUpdate = (updates: {
    tags?: string[];
  }) => {
    let nextContent = content;
    const delimiter = documentModel.delimiter || '+++';

    if (updates.tags) {
      nextContent = setFrontmatterStringListField(nextContent, 'tags', updates.tags, delimiter) || nextContent;
    }

    if (nextContent !== content) {
      setContent(nextContent);
    }
  };

  const submitCreate = async (event: FormEvent) => {
    event.preventDefault();
    const parsed = splitFrontmatterDocument(content);
    const cleanTitle = readFrontmatterStringField(parsed.frontmatter, 'title').trim();
    if (!cleanTitle) {
      setError('Title is required.');
      return;
    }
    try {
      setCreating(true);
      await onCreateSpec(cleanTitle, content);
      setCreateOpen(false);
      setContent(createInitialSpecDocument());
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

  useEffect(() => {
    if (createOpen) return;
    setContent(createInitialSpecDocument());
  }, [createOpen]);

  return (
    <PageFrame>
      <PageHeader
        title="Specs"
        description="Refine intent, then turn specs into actionable issues."
        actions={
          <div className="flex items-center gap-2">
            <Tooltip>
              <TooltipTrigger asChild>
                <div>
                  <TemplateEditorButton kind="spec" />
                </div>
              </TooltipTrigger>
              <TooltipContent side="bottom">Refine the markdown templates used for new specifications.</TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger asChild>
                <Button className="gap-2" onClick={() => setCreateOpen(true)}>
                  <Plus className="size-4" />
                  New Spec
                </Button>
              </TooltipTrigger>
              <TooltipContent side="bottom">Create a new technical specification document.</TooltipContent>
            </Tooltip>
          </div>
        }
      />

      {specs.length === 0 ? (
        <EmptyState
          icon={<FileCode2 className="size-4" />}
          title="No specs yet"
          description="Specs define scope, decisions, and delivery criteria."
          action={
            <Button onClick={() => setCreateOpen(true)}>
              <Plus className="mr-2 size-4" />
              Create First Spec
            </Button>
          }
        />
      ) : (
        <Card size="sm">
          <CardHeader className="pb-2">
            <div className="flex items-start justify-between gap-3">
              <div>
                <CardTitle className="text-sm">Specification Library</CardTitle>
                <CardDescription>
                  {specs.length} spec{specs.length !== 1 ? 's' : ''} in this project
                </CardDescription>
              </div>
              <div className="flex items-center gap-2">
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Input
                      value={search}
                      onChange={(event) => setSearch(event.target.value)}
                      placeholder="Search specs"
                      className="h-8 w-[220px]"
                    />
                  </TooltipTrigger>
                  <TooltipContent side="top">Filter specifications by title or filename.</TooltipContent>
                </Tooltip>

                <Tooltip delayDuration={300}>
                  <TooltipTrigger asChild>
                    <div className="w-[180px]">
                      <Select value={sortBy} onValueChange={(value) => setSortBy(value as SpecSort)}>
                        <SelectTrigger size="sm" className="w-full">
                          <SelectValue>
                            {SPEC_SORT_OPTIONS.find((option) => option.value === sortBy)?.label}
                          </SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                          {SPEC_SORT_OPTIONS.map((option) => (
                            <SelectItem key={option.value} value={option.value}>
                              {option.label}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                  </TooltipTrigger>
                  <TooltipContent side="top">Sort specifications by name or date.</TooltipContent>
                </Tooltip>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-2">
            {sortedSpecs.map((spec) => (
              <div
                key={spec.path}
                className="hover:bg-muted/40 grid gap-2 rounded-md border p-3 transition-colors md:grid-cols-[1fr_auto] md:items-center"
                title={spec.path}
              >
                <div className="min-w-0">
                  <p className="truncate text-sm font-medium">{spec.spec.metadata.title}</p>
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
            <SpecHeaderMetadata
              fileName="new-spec.md"
              tags={currentTags}
              isEditing={true}
              onUpdate={handleMetadataUpdate}
              tagSuggestions={tagSuggestions}
            />
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
              <Alert variant="destructive">
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            <MarkdownEditor
              label="Content"
              value={content}
              onChange={(next) => {
                setContent(next);
                setError(null);
              }}
              placeholder="# Alpha Spec"
              rows={22}
              defaultMode="doc"
            />
          </form>
        </DetailSheet>
      )}
    </PageFrame>
  );
}
