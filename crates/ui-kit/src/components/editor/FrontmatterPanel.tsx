import { useEffect, useMemo, useState } from 'react';
import { ChevronDown, ChevronUp, Plus, Tag } from 'lucide-react';
import { Badge } from '../badge';
import { Button } from '../button';
import { Input } from '../input';
import { Textarea } from '../textarea';
import { FacetedFilter } from '../faceted-filter';
import { AutocompleteInput } from '../autocomplete-input';
import { cn } from '@/lib/utils';
import {
    FrontmatterDelimiter,
    readFrontmatterSummary,
    setFrontmatterStringField,
    setFrontmatterStringListField,
} from './frontmatter';

const DEFAULT_STATUS_OPTIONS = ['draft', 'backlog', 'in-progress', 'review', 'done'];

export interface FrontmatterPanelProps {
    frontmatter: string | null;
    delimiter: FrontmatterDelimiter | null;
    className?: string;
    statusOptions?: string[];
    specSuggestions?: { id: string; title: string }[];
    onChange: (frontmatter: string | null, delimiter: FrontmatterDelimiter) => void;
}

function createStarterFrontmatter(delimiter: FrontmatterDelimiter): string {
    if (delimiter === '---') return 'status: "draft"\ntags: ["editor"]';
    return 'status = "draft"\ntags = ["editor"]';
}

export default function FrontmatterPanel({
    frontmatter,
    delimiter,
    className,
    statusOptions,
    specSuggestions = [],
    onChange,
}: FrontmatterPanelProps) {
    const [expanded, setExpanded] = useState(false);
    const [rawText, setRawText] = useState(frontmatter ?? '');

    const currentDelimiter: FrontmatterDelimiter = delimiter ?? '+++';
    const summary = useMemo(() => readFrontmatterSummary(frontmatter), [frontmatter]);
    const availableStatuses = useMemo(() => {
        const fromProps = statusOptions ?? [];
        const merged = new Set([...DEFAULT_STATUS_OPTIONS, ...fromProps]);
        if (summary.status) merged.add(summary.status);
        return Array.from(merged);
    }, [statusOptions, summary.status]);

    useEffect(() => {
        setRawText(frontmatter ?? '');
    }, [frontmatter]);

    const applyUpdate = (next: string | null, nextDelimiter: FrontmatterDelimiter = currentDelimiter) => {
        const trimmed = next?.trim() ?? '';
        onChange(trimmed ? trimmed : null, nextDelimiter);
    };

    const updateTitle = (title: string) => {
        const next = setFrontmatterStringField(frontmatter, 'title', title, currentDelimiter);
        applyUpdate(next);
    };

    const updateStatus = (status: string) => {
        const next = setFrontmatterStringField(frontmatter, 'status', status, currentDelimiter);
        applyUpdate(next);
    };

    const updateTags = (tags: string[]) => {
        const next = setFrontmatterStringListField(frontmatter, 'tags', tags, currentDelimiter);
        applyUpdate(next);
    };

    const updateSpecs = (specs: string[]) => {
        const next = setFrontmatterStringListField(frontmatter, 'specs', specs, currentDelimiter);
        applyUpdate(next);
    };

    if (!frontmatter) {
        return (
            <section className={cn('rounded-md border border-dashed bg-muted/20 p-2.5', className)}>
                <div className="flex items-center justify-between gap-2">
                    <p className="text-muted-foreground text-xs">No metadata yet.</p>
                    <Button
                        type="button"
                        variant="outline"
                        size="xs"
                        onClick={() => {
                            applyUpdate(createStarterFrontmatter(currentDelimiter));
                            setExpanded(true);
                        }}
                    >
                        <Plus className="size-3.5" />
                        Add Metadata
                    </Button>
                </div>
            </section>
        );
    }

    return (
        <section className={cn('rounded-md border bg-card', className)}>
            <div className="flex flex-wrap items-center gap-1.5 border-b px-2 py-1.5">
                <Badge variant="outline" className="text-[11px] uppercase tracking-wide">
                    Metadata
                </Badge>
                {summary.status && (
                    <Badge variant="secondary" className="text-[11px]">
                        {summary.status}
                    </Badge>
                )}
                {summary.tags.slice(0, 4).map((tag) => (
                    <Badge key={tag} variant="outline" className="text-[11px]">
                        <Tag className="size-3" />
                        {tag}
                    </Badge>
                ))}
                {summary.title && (
                    <span className="text-muted-foreground truncate text-xs">
                        {summary.title}
                    </span>
                )}
                <div className="ml-auto flex items-center gap-1">
                    <Badge variant="outline" className="font-mono text-[10px]">
                        {currentDelimiter}
                    </Badge>
                    <Button type="button" variant="ghost" size="icon-sm" onClick={() => setExpanded((prev) => !prev)}>
                        {expanded ? <ChevronUp className="size-3.5" /> : <ChevronDown className="size-3.5" />}
                    </Button>
                </div>
            </div>

            {expanded && (
                <div className="grid gap-3 p-2.5">
                    <div className="grid gap-2 md:grid-cols-2">
                        <div className="space-y-1">
                            <label className="text-muted-foreground text-xs font-medium uppercase tracking-wide">Title</label>
                            <Input
                                value={summary.title}
                                placeholder="Document title"
                                className="h-8"
                                onChange={(event) => updateTitle(event.target.value)}
                            />
                        </div>
                        <div className="space-y-1">
                            <label className="text-muted-foreground text-xs font-medium uppercase tracking-wide">Status</label>
                            <AutocompleteInput
                                value={summary.status}
                                options={availableStatuses.map((value) => ({ value }))}
                                placeholder="Status"
                                className="h-8"
                                noResultsText="No status matches."
                                onValueChange={updateStatus}
                            />
                        </div>
                    </div>

                    <div className="space-y-1">
                        <label className="text-muted-foreground text-xs font-medium uppercase tracking-wide">
                            Tags {summary.tags.length ? `(${summary.tags.length})` : ''}
                        </label>
                        <div className="w-full">
                            <FacetedFilter
                                title="Add tag"
                                options={[]}
                                selectedValues={summary.tags}
                                onSelectionChange={updateTags}
                                allowNew
                                onAddNew={(tag) => updateTags([...summary.tags, tag])}
                            />
                        </div>
                    </div>

                    <div className="space-y-1">
                        <label className="text-muted-foreground text-xs font-medium uppercase tracking-wide">
                            Linked Specs {summary.specs.length ? `(${summary.specs.length})` : ''}
                        </label>
                        <div className="w-full">
                            <FacetedFilter
                                title="Link spec"
                                options={specSuggestions.map(s => ({ value: s.id, label: s.title }))}
                                selectedValues={summary.specs}
                                onSelectionChange={updateSpecs}
                            />
                        </div>
                    </div>

                    <div className="space-y-1">
                        <label className="text-muted-foreground text-xs font-medium uppercase tracking-wide">Raw Metadata</label>
                        <Textarea
                            rows={8}
                            value={rawText}
                            className="font-mono text-xs leading-5"
                            onChange={(event) => {
                                const next = event.target.value;
                                setRawText(next);
                                applyUpdate(next);
                            }}
                        />
                    </div>

                    <div className="flex justify-end">
                        <Button
                            type="button"
                            variant="ghost"
                            size="xs"
                            className="text-destructive"
                            onClick={() => applyUpdate(null)}
                        >
                            Remove Metadata
                        </Button>
                    </div>
                </div>
            )}
        </section>
    );
}
