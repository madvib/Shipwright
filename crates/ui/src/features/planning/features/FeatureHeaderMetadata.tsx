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
    Button,
    Badge,
    AutocompleteInput,
    FacetedFilter,
} from '@ship/ui';
import { cn } from '@/lib/utils';
import { formatStatusLabel } from '@/features/planning/common/hub/utils/featureMetrics';
import { BaseMetadataHeader } from '../common/BaseMetadataHeader';
import { MetadataPopover } from '../common/MetadataPopover';

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
        <BaseMetadataHeader>
            {/* Status Popover */}
            <MetadataPopover
                icon={StatusIcon}
                label={formatStatusLabel(status)}
                title="Change Status"
                triggerClassName={cn(
                    status === 'in-progress' && "[&_svg]:text-blue-500",
                    status === 'done' && "[&_svg]:text-emerald-500",
                    status === 'blocked' && "[&_svg]:text-destructive",
                    status === 'todo' && "[&_svg]:text-amber-500"
                )}
                contentClassName="w-48 p-2"
            >
                <div className="space-y-1">
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
            </MetadataPopover>

            {/* Release Popover */}
            <MetadataPopover
                icon={Package}
                label={releaseId || 'No Release'}
                title="Linked Release"
                action={releaseId && onNavigate && (
                    <Button variant="link" size="xs" className="h-auto p-0 text-[10px]" onClick={() => onNavigate(releaseId, 'release')}>
                        View Release
                    </Button>
                )}
            >
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
            </MetadataPopover>

            {/* Spec Popover */}
            <MetadataPopover
                icon={Shapes}
                label={specId || 'No Spec'}
                title="Linked Specification"
                action={specId && onNavigate && (
                    <Button variant="link" size="xs" className="h-auto p-0 text-[10px]" onClick={() => onNavigate(specId, 'spec')}>
                        View Spec
                    </Button>
                )}
            >
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
            </MetadataPopover>

            {/* Tags Popover */}
            <MetadataPopover
                icon={Tag}
                label={tags.length > 0 ? `${tags.length} Tag${tags.length === 1 ? '' : 's'}` : 'No tags'}
                title="Tags"
                triggerClassName="shrink-0"
                contentClassName="w-64 p-3"
            >
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
            </MetadataPopover>
        </BaseMetadataHeader>
    );
}
