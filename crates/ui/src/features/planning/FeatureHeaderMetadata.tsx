import { useState } from 'react';
import {
    Shapes,
    Tag,
    CheckCircle2,
    Circle,
    Clock,
    AlertTriangle,
    Package,
    FolderKanban
} from 'lucide-react';
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
    Button,
    Badge,
    AutocompleteInput,
    FacetedFilter,
} from '@ship/ui';
import { cn } from '@/lib/utils';
import { formatStatusLabel } from '@/features/planning/hub/utils/featureMetrics';

interface FeatureHeaderMetadataProps {
    status: string;
    releaseId?: string;
    specId?: string;
    tags?: string[];
    isEditing: boolean;
    onUpdate: (updates: {
        status?: string;
        release_id?: string;
        spec_id?: string;
        tags?: string[];
    }) => void;
    releaseSuggestions?: string[];
    specSuggestions?: string[];
    tagSuggestions?: string[];
    onNavigate?: (id: string, type: 'release' | 'spec') => void;
}

const STATUS_OPTIONS = [
    { value: 'backlog', label: 'Backlog', icon: FolderKanban },
    { value: 'todo', label: 'Todo', icon: Circle },
    { value: 'in-progress', label: 'In Progress', icon: Clock },
    { value: 'done', label: 'Done', icon: CheckCircle2 },
    { value: 'blocked', label: 'Blocked', icon: AlertTriangle },
];

export function FeatureHeaderMetadata({
    status,
    releaseId,
    specId,
    tags = [],
    isEditing,
    onUpdate,
    releaseSuggestions = [],
    specSuggestions = [],
    tagSuggestions = [],
    onNavigate,
}: FeatureHeaderMetadataProps) {
    const currentStatus = STATUS_OPTIONS.find(s => s.value === status) || STATUS_OPTIONS[1];
    const StatusIcon = currentStatus.icon;

    const [releaseInput, setReleaseInput] = useState(releaseId || '');
    const [specInput, setSpecInput] = useState(specId || '');

    return (
        <div className="flex flex-nowrap items-center gap-4 text-xs text-muted-foreground overflow-hidden">
            {/* Status Popover */}
            <Popover>
                <PopoverTrigger
                    render={
                        <Button
                            variant="ghost"
                            size="xs"
                            className="h-7 gap-1.5 px-2 text-xs font-normal text-muted-foreground hover:bg-accent/50"
                        />
                    }
                >
                    <StatusIcon className={cn(
                        "size-3.5",
                        status === 'in-progress' && "text-blue-500",
                        status === 'done' && "text-emerald-500",
                        status === 'blocked' && "text-destructive",
                        status === 'todo' && "text-amber-500"
                    )} />
                    <span>{formatStatusLabel(status)}</span>
                </PopoverTrigger>
                <PopoverContent align="start" className="w-48 p-2">
                    <div className="space-y-1">
                        <p className="px-2 py-1.5 text-[10px] font-bold uppercase tracking-wider text-muted-foreground">Change Status</p>
                        {STATUS_OPTIONS.map((opt) => (
                            <Button
                                key={opt.value}
                                variant="ghost"
                                size="xs"
                                className={cn(
                                    "w-full justify-start gap-2 h-8 font-normal",
                                    status === opt.value && "bg-accent text-accent-foreground"
                                )}
                                onClick={() => isEditing && onUpdate({ status: opt.value })}
                                disabled={!isEditing}
                            >
                                <opt.icon className="size-3.5" />
                                {opt.label}
                            </Button>
                        ))}
                    </div>
                </PopoverContent>
            </Popover>

            <span className="text-muted-foreground/30 px-1">|</span>

            {/* Release Popover */}
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
                    <Package className="size-3.5 shrink-0" />
                    <span className="truncate">{releaseId || 'No Release'}</span>
                </PopoverTrigger>
                <PopoverContent align="start" className="w-80 p-2">
                    <div className="space-y-3">
                        <div className="flex items-center justify-between">
                            <p className="px-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground">Linked Release</p>
                            {releaseId && onNavigate && (
                                <Button variant="link" size="xs" className="h-auto p-0 text-[10px]" onClick={() => onNavigate(releaseId, 'release')}>
                                    View Release
                                </Button>
                            )}
                        </div>
                        {isEditing ? (
                            <AutocompleteInput
                                value={releaseInput}
                                onValueChange={setReleaseInput}
                                options={releaseSuggestions.map(id => ({ value: id, label: id }))}
                                placeholder="Search releases..."
                                onCommit={(val) => onUpdate({ release_id: val || undefined })}
                            />
                        ) : (
                            <div className="rounded-md border bg-muted/20 p-2 mx-1">
                                <p className="text-sm font-medium">{releaseId || 'None'}</p>
                            </div>
                        )}
                    </div>
                </PopoverContent>
            </Popover>

            <span className="text-muted-foreground/30 px-1">|</span>

            {/* Spec Popover */}
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
                    <span className="truncate">{specId || 'No Spec'}</span>
                </PopoverTrigger>
                <PopoverContent align="start" className="w-80 p-2">
                    <div className="space-y-3">
                        <div className="flex items-center justify-between">
                            <p className="px-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground">Linked Specification</p>
                            {specId && onNavigate && (
                                <Button variant="link" size="xs" className="h-auto p-0 text-[10px]" onClick={() => onNavigate(specId, 'spec')}>
                                    View Spec
                                </Button>
                            )}
                        </div>
                        {isEditing ? (
                            <AutocompleteInput
                                value={specInput}
                                onValueChange={setSpecInput}
                                options={specSuggestions.map(id => ({ value: id, label: id }))}
                                placeholder="Search specs..."
                                onCommit={(val) => onUpdate({ spec_id: val || undefined })}
                            />
                        ) : (
                            <div className="rounded-md border bg-muted/20 p-2 mx-1">
                                <p className="text-sm font-medium">{specId || 'None'}</p>
                            </div>
                        )}
                    </div>
                </PopoverContent>
            </Popover>

            <span className="text-muted-foreground/30 px-1">|</span>

            {/* Tags Popover */}
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
                <PopoverContent align="start" className="w-64 p-3 text-left">
                    <div className="space-y-3">
                        <p className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground">Tags</p>
                        <div className="flex flex-wrap gap-1.5">
                            {tags.map((tag) => (
                                <Badge key={tag} variant="secondary" className="h-5 px-1.5 text-[10px] font-normal uppercase">
                                    {tag}
                                </Badge>
                            ))}
                            {tags.length === 0 && (
                                <span className="text-xs text-muted-foreground italic">No tags</span>
                            )}
                        </div>
                        {isEditing && (
                            <FacetedFilter
                                title="Edit Tags"
                                options={tagSuggestions.map(t => ({ value: t, label: t }))}
                                selectedValues={tags}
                                onSelectionChange={(next) => onUpdate({ tags: next })}
                                allowNew
                                onAddNew={(tag) => onUpdate({ tags: [...tags, tag] })}
                            />
                        )}
                    </div>
                </PopoverContent>
            </Popover>
        </div>
    );
}
