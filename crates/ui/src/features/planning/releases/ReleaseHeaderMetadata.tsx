import {
    Calendar as CalendarIcon,
    Package,
    Tag,
    CheckCircle2,
    Circle,
    Clock,
    Archive
} from 'lucide-react';
import {
    Button,
    Badge,
    DatePicker,
    FacetedFilter,
    Tooltip,
    TooltipTrigger,
    TooltipContent,
} from '@ship/ui';
import { formatDate } from '@/lib/date';
import { cn } from '@/lib/utils';
import { formatStatusLabel } from '@/features/planning/common/hub/utils/featureMetrics';
import { BaseMetadataHeader } from '../common/BaseMetadataHeader';
import { MetadataPopover } from '../common/MetadataPopover';

interface ReleaseHeaderMetadataProps {
    version: string;
    status: string;
    targetDate?: string;
    tags?: string[];
    isEditing: boolean;
    onUpdate: (updates: {
        version?: string;
        status?: string;
        target_date?: string;
        tags?: string[];
    }) => void;
    tagSuggestions?: string[];
}

const STATUS_OPTIONS = [
    { value: 'planned', label: 'Planned', icon: Circle },
    { value: 'active', label: 'Active', icon: Clock },
    { value: 'shipped', label: 'Shipped', icon: CheckCircle2 },
    { value: 'archived', label: 'Archived', icon: Archive },
];

export function ReleaseHeaderMetadata({
    version,
    status,
    targetDate,
    tags = [],
    isEditing,
    onUpdate,
    tagSuggestions = [],
}: ReleaseHeaderMetadataProps) {

    const currentStatus = STATUS_OPTIONS.find(s => s.value === status) || STATUS_OPTIONS[0];
    const StatusIcon = currentStatus.icon;

    return (
        <BaseMetadataHeader>
            {/* Version Badge */}
            <div className="flex items-center gap-1.5 shrink-0">
                <Package className="size-3.5" />
                <span className="font-bold text-foreground">{version}</span>
            </div>

            {/* Status Popover */}
            <Tooltip>
                <TooltipTrigger asChild>
                    <div>
                        <MetadataPopover
                            icon={StatusIcon}
                            label={formatStatusLabel(status)}
                            title="Change Status"
                            triggerClassName={cn(
                                status === 'active' && "[&_svg]:text-blue-500",
                                status === 'shipped' && "[&_svg]:text-emerald-500",
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
                                        onClick={() => isEditing && onUpdate({ status: opt.value })}
                                        disabled={!isEditing}
                                    >
                                        <opt.icon className="size-3.5" />
                                        {opt.label}
                                    </Button>
                                ))}
                            </div>
                        </MetadataPopover>
                    </div>
                </TooltipTrigger>
                <TooltipContent side="bottom">View and manage release status.</TooltipContent>
            </Tooltip>

            {/* Date Picker */}
            <div className="flex items-center shrink-0">
                {isEditing ? (
                    <DatePicker
                        value={targetDate || ''}
                        onValueChange={(next) => onUpdate({ target_date: next })}
                        className="h-7 w-auto border-none bg-transparent p-0 hover:bg-accent/50 text-muted-foreground transition-colors"
                    />
                ) : (
                    <div className="flex items-center gap-1.5">
                        <CalendarIcon className="size-3.5" />
                        <span>{targetDate ? formatDate(targetDate) : 'No target date'}</span>
                    </div>
                )}
            </div>

            {/* Tags Popover */}
            <Tooltip>
                <TooltipTrigger asChild>
                    <div>
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
                    </div>
                </TooltipTrigger>
                <TooltipContent side="bottom">View and manage tags for this release.</TooltipContent>
            </Tooltip>
        </BaseMetadataHeader>
    );
}
