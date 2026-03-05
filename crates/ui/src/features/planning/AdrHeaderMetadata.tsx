import { useState } from 'react';
import { ADR } from '@/bindings';
import { formatDate } from '@/lib/date';
import { Shapes, GitBranch, Calendar as CalendarIcon, Tag } from 'lucide-react';
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
    Button,
    DatePicker,
    AutocompleteInput,
    Badge,
} from '@ship/ui';
import { cn } from '@/lib/utils';

interface AdrHeaderMetadataProps {
    adr: ADR;
    onChange: (next: ADR) => void;
    specSuggestions: { id: string; title: string }[];
    adrSuggestions: { id: string; title: string }[];
    tagSuggestions: string[];
    isEditing: boolean;
    onNavigate?: (type: 'spec' | 'adr', id: string) => void;
}

export function AdrHeaderMetadata({
    adr,
    onChange,
    specSuggestions,
    adrSuggestions,
    tagSuggestions,
    isEditing,
    onNavigate,
}: AdrHeaderMetadataProps) {
    const { date, spec_id, supersedes_id, tags = [] } = adr.metadata;

    const specTitle = specSuggestions.find(s => s.id === spec_id)?.title || spec_id;
    const adrTitle = adrSuggestions.find(a => a.id === supersedes_id)?.title || supersedes_id;
    const [specInput, setSpecInput] = useState(spec_id || '');
    const [lineageInput, setLineageInput] = useState(supersedes_id || '');
    const [tagInput, setTagInput] = useState('');

    const updateMetadata = (updates: Partial<ADR['metadata']>) => {
        onChange({
            ...adr,
            metadata: {
                ...adr.metadata,
                ...updates,
            },
        });
    };

    if (!isEditing) {
        return (
            <div className="flex flex-nowrap items-center gap-4 text-xs text-muted-foreground overflow-hidden">
                <div className="flex items-center gap-1.5 shrink-0">
                    <CalendarIcon className="size-3.5" />
                    <span>{date ? formatDate(date) : 'No date'}</span>
                </div>
                <div className="flex items-center gap-1.5 shrink-0">
                    <Shapes className="size-3.5" />
                    <span className="truncate max-w-[150px]">{spec_id || 'No linked spec'}</span>
                </div>
                <div className="flex items-center gap-1.5 shrink-0">
                    <GitBranch className="size-3.5" />
                    <span className="truncate max-w-[180px]">{supersedes_id ? `Supersedes ${supersedes_id}` : 'No lineage link'}</span>
                </div>
                {tags.length > 0 && (
                    <div className="flex items-center gap-1.5 shrink-0 overflow-hidden">
                        <Tag className="size-3.5" />
                        <div className="flex gap-1 overflow-hidden">
                            {tags.map((tag) => (
                                <Badge key={tag} variant="secondary" className="h-4 px-1 text-[9px] font-normal uppercase shrink-0">
                                    {tag}
                                </Badge>
                            ))}
                        </div>
                    </div>
                )}
            </div>
        );
    }

    return (
        <div className="flex flex-nowrap items-center gap-1 overflow-hidden">
            {/* Date Picker Popover */}
            <DatePicker
                value={date || ''}
                onValueChange={(next) => updateMetadata({ date: next })}
                className={cn(
                    "h-7 w-auto border-none bg-transparent p-0 hover:bg-accent/50 text-muted-foreground transition-colors",
                    !isEditing && "pointer-events-none"
                )}
            />

            <span className="text-muted-foreground/30 px-1">|</span>

            {/* Spec Autocomplete Popover */}
            <Popover>
                <PopoverTrigger
                    render={
                        <Button
                            variant="ghost"
                            size="xs"
                            className="h-7 gap-1.5 px-2 text-xs font-normal text-muted-foreground hover:bg-accent/50 max-w-[200px]"
                        />
                    }
                >
                    <Shapes className="size-3.5 shrink-0" />
                    <span className="truncate">{specTitle || 'No linked spec'}</span>
                </PopoverTrigger>
                <PopoverContent align="start" className="w-80 p-2">
                    <div className="space-y-3">
                        <div className="flex items-center justify-between">
                            <p className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground">Linked Spec</p>
                            {spec_id && onNavigate && (
                                <Button
                                    variant="link"
                                    size="xs"
                                    className="h-auto p-0 text-[10px]"
                                    onClick={() => onNavigate('spec', spec_id)}
                                >
                                    View Spec
                                </Button>
                            )}
                        </div>
                        {isEditing ? (
                            <AutocompleteInput
                                value={specInput}
                                onValueChange={setSpecInput}
                                options={specSuggestions.map((s) => ({ value: s.id, label: s.title }))}
                                placeholder="Search specs..."
                                onCommit={(val) => updateMetadata({ spec_id: val || null })}
                            />
                        ) : (
                            <div className="rounded-md border bg-muted/20 p-2">
                                <p className="text-sm font-medium">{specTitle || 'None'}</p>
                                <p className="text-[10px] text-muted-foreground font-mono mt-0.5">{spec_id}</p>
                            </div>
                        )}
                    </div>
                </PopoverContent>
            </Popover>

            <span className="text-muted-foreground/30 px-1">|</span>

            {/* Supersedes Autocomplete Popover */}
            <Popover>
                <PopoverTrigger
                    render={
                        <Button
                            variant="ghost"
                            size="xs"
                            className="h-7 gap-1.5 px-2 text-xs font-normal text-muted-foreground hover:bg-accent/50 max-w-[200px]"
                        />
                    }
                >
                    <GitBranch className="size-3.5 shrink-0" />
                    <span className="truncate">{supersedes_id ? `Supersedes: ${adrTitle}` : 'No lineage link'}</span>
                </PopoverTrigger>
                <PopoverContent align="start" className="w-80 p-2">
                    <div className="space-y-3">
                        <div className="flex items-center justify-between">
                            <p className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground">Supersedes</p>
                            {supersedes_id && onNavigate && (
                                <Button
                                    variant="link"
                                    size="xs"
                                    className="h-auto p-0 text-[10px]"
                                    onClick={() => onNavigate('adr', supersedes_id)}
                                >
                                    View ADR
                                </Button>
                            )}
                        </div>
                        {isEditing ? (
                            <AutocompleteInput
                                value={lineageInput}
                                onValueChange={setLineageInput}
                                options={adrSuggestions.map((a) => ({ value: a.id, label: a.title }))}
                                placeholder="Search ADRs..."
                                onCommit={(val) => updateMetadata({ supersedes_id: val || null })}
                            />
                        ) : (
                            <div className="rounded-md border bg-muted/20 p-2">
                                <p className="text-sm font-medium">{adrTitle || 'None'}</p>
                                <p className="text-[10px] text-muted-foreground font-mono mt-0.5">{supersedes_id}</p>
                            </div>
                        )}
                    </div>
                </PopoverContent>
            </Popover>

            <span className="text-muted-foreground/30 px-1">|</span>

            {/* Tags Editor Popover */}
            <Popover>
                <PopoverTrigger
                    render={
                        <Button
                            variant="ghost"
                            size="xs"
                            className="h-7 gap-1.5 px-2 text-xs font-normal text-muted-foreground hover:bg-accent/50 shrink-0"
                        />
                    }
                >
                    <Tag className="size-3.5" />
                    {tags.length > 0 ? `${tags.length} Tag${tags.length === 1 ? '' : 's'}` : 'No tags'}
                </PopoverTrigger>
                <PopoverContent align="start" className="w-64 p-3">
                    <div className="space-y-3">
                        <p className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground">Tags</p>
                        <div className="flex flex-wrap gap-1.5">
                            {tags.map((tag) => (
                                <Badge key={tag} variant="secondary" className="h-5 px-1.5 text-[10px] font-normal uppercase">
                                    {tag}
                                    {isEditing && (
                                        <button
                                            className="ml-1 hover:text-destructive"
                                            onClick={() => updateMetadata({ tags: tags.filter((t) => t !== tag) })}
                                        >
                                            ×
                                        </button>
                                    )}
                                </Badge>
                            ))}
                            {tags.length === 0 && !isEditing && (
                                <span className="text-xs text-muted-foreground italic">No tags</span>
                            )}
                        </div>
                        {isEditing && (
                            <AutocompleteInput
                                value={tagInput}
                                onValueChange={setTagInput}
                                options={tagSuggestions.filter(t => !tags.includes(t)).map((t) => ({ value: t }))}
                                placeholder="Add tag..."
                                onCommit={(val) => {
                                    if (val && !tags.includes(val)) {
                                        updateMetadata({ tags: [...tags, val] });
                                        setTagInput('');
                                    }
                                }}
                            />
                        )}
                    </div>
                </PopoverContent>
            </Popover>
        </div>
    );
}
