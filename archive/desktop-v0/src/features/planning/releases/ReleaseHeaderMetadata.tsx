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
    Input,
    Tooltip,
    TooltipTrigger,
    TooltipContent,
} from '@ship/primitives';
import { formatDate } from '@/lib/date';
import { cn } from '@/lib/utils';
import { formatStatusLabel } from '@/features/planning/common/hub/utils/featureMetrics';
import { BaseMetadataHeader } from '../common/BaseMetadataHeader';
import { MetadataPopover } from '../common/MetadataPopover';

interface ReleaseHeaderMetadataProps {
    version: string;
    status: string;
    supported?: boolean;
    targetDate?: string;
    tags?: string[];
    isEditing: boolean;
    onUpdate: (updates: {
        version?: string;
        status?: string;
        supported?: boolean;
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
    supported = false,
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
            {/* Version */}
            <div className="flex items-center gap-1.5 shrink-0">
                <Package className="size-3.5" />
                {isEditing ? (
                    <Input
                        value={version}
                        onChange={(event) => onUpdate({ version: event.target.value })}
                        className="h-7 w-[11rem] border-border/40 bg-transparent px-2 text-xs font-semibold"
                        placeholder="v0.1.1-alpha"
                    />
                ) : (
                    <span className="font-bold text-foreground">{version}</span>
                )}
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

            {/* Support Status */}
            <Tooltip>
                <TooltipTrigger asChild>
                    <div>
                        <MetadataPopover
                            icon={supported ? CheckCircle2 : Circle}
                            label={supported ? 'Supported' : 'Unsupported'}
                            title="Support Status"
                            triggerClassName={cn(
                                supported
                                    ? "[&_svg]:text-emerald-500"
                                    : "[&_svg]:text-muted-foreground"
                            )}
                            contentClassName="w-48 p-2"
                        >
                            <div className="space-y-1">
                                <Button
                                    variant="ghost"
                                    size="xs"
                                    className={cn(
                                        "w-full justify-start gap-2 h-8 font-normal",
                                        supported && "bg-accent text-accent-foreground"
                                    )}
                                    onClick={() => isEditing && onUpdate({ supported: true })}
                                    disabled={!isEditing}
                                >
                                    <CheckCircle2 className="size-3.5" />
                                    Supported
                                </Button>
                                <Button
                                    variant="ghost"
                                    size="xs"
                                    className={cn(
                                        "w-full justify-start gap-2 h-8 font-normal",
                                        !supported && "bg-accent text-accent-foreground"
                                    )}
                                    onClick={() => isEditing && onUpdate({ supported: false })}
                                    disabled={!isEditing}
                                >
                                    <Circle className="size-3.5" />
                                    Unsupported
                                </Button>
                            </div>
                        </MetadataPopover>
                    </div>
                </TooltipTrigger>
                <TooltipContent side="bottom">Set maintenance/support intent for this release.</TooltipContent>
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
