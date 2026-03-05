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
    Popover,
    PopoverContent,
    PopoverTrigger,
    Button,
    Badge,
    DatePicker,
    FacetedFilter,
} from '@ship/ui';
import { formatDate } from '@/lib/date';
import { cn } from '@/lib/utils';
import { formatStatusLabel } from '@/features/planning/hub/utils/featureMetrics';

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
        <div className="flex flex-nowrap items-center gap-4 text-xs text-muted-foreground overflow-hidden">
            {/* Version Badge */}
            <div className="flex items-center gap-1.5 shrink-0">
                <Package className="size-3.5" />
                <span className="font-bold text-foreground">{version}</span>
            </div>

            <span className="text-muted-foreground/30 px-1">|</span>

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
                        status === 'active' && "text-blue-500",
                        status === 'shipped' && "text-emerald-500",
                        status === 'planned' && "text-amber-500"
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

            {/* Date Picker */}
            <div className="flex items-center gap-1.5 shrink-0">
                <CalendarIcon className="size-3.5" />
                {isEditing ? (
                    <DatePicker
                        value={targetDate || ''}
                        onValueChange={(next) => onUpdate({ target_date: next })}
                        className="h-7 w-auto border-none bg-transparent p-0 hover:bg-accent/50 text-muted-foreground transition-colors"
                    />
                ) : (
                    <span>{targetDate ? formatDate(targetDate) : 'No target date'}</span>
                )}
            </div>

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
