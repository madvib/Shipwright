import { useState } from 'react';
import {
    Tag,
    CheckCircle2,
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
} from '@ship/primitives';
import { cn } from '@/lib/utils';
import { formatStatusLabel } from '@/features/planning/common/hub/utils/featureMetrics';
import { BaseMetadataHeader } from '../common/BaseMetadataHeader';
import { MetadataPopover } from '../common/MetadataPopover';

interface FeatureHeaderMetadataProps {
    status: string;
    releaseId?: string;
    tags?: string[];
    isEditing: boolean;
    onUpdate: (updates: {
        release_id?: string;
        tags?: string[];
    }) => void;
    onStatusTransition?: (status: string) => Promise<void> | void;
    releaseSuggestions?: string[];
    tagSuggestions?: string[];
    onNavigate?: (id: string, type: 'release') => void;
}

const STATUS_OPTIONS = [
    { value: 'planned', label: 'Planned', icon: FolderKanban },
    { value: 'in-progress', label: 'In Progress', icon: Clock },
    { value: 'implemented', label: 'Implemented', icon: CheckCircle2 },
    { value: 'deprecated', label: 'Deprecated', icon: AlertTriangle },
];

export function FeatureHeaderMetadata({
    status,
    releaseId,
    tags = [],
    isEditing,
    onUpdate,
    onStatusTransition,
    releaseSuggestions = [],
    tagSuggestions = [],
    onNavigate,
}: FeatureHeaderMetadataProps) {
    const currentStatus = STATUS_OPTIONS.find(s => s.value === status) || STATUS_OPTIONS[1];
    const StatusIcon = currentStatus.icon;

    const [releaseInput, setReleaseInput] = useState(releaseId || '');

    const canTransition = (nextStatus: string) => {
        if (nextStatus === status) {
            return false;
        }
        if (status === 'planned') {
            return nextStatus === 'in-progress';
        }
        if (status === 'in-progress') {
            return nextStatus === 'implemented';
        }
        return false;
    };

    const handleStatusClick = (nextStatus: string) => {
        if (!canTransition(nextStatus) || !onStatusTransition) {
            return;
        }
        void Promise.resolve(onStatusTransition(nextStatus)).catch(() => {
            // Errors are surfaced by shared workspace error state in action hooks.
        });
    };

    return (
        <BaseMetadataHeader>
            {/* Status Popover */}
            <MetadataPopover
                icon={StatusIcon}
                label={formatStatusLabel(status)}
                title="Change Status"
                triggerClassName={cn(
                    status === 'in-progress' && "[&_svg]:text-blue-500",
                    status === 'implemented' && "[&_svg]:text-emerald-500",
                    status === 'deprecated' && "[&_svg]:text-destructive",
                    status === 'planned' && "[&_svg]:text-amber-500"
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
                            onClick={() => handleStatusClick(opt.value)}
                            disabled={!canTransition(opt.value)}
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


            {/* Tags Popover */}
            {(isEditing || tags.length > 0) && (
                <MetadataPopover
                    icon={Tag}
                    label={`${tags.length} Tag${tags.length === 1 ? '' : 's'}`}
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
            )}
        </BaseMetadataHeader>
    );
}
