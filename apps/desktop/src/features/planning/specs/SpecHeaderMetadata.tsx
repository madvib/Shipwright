import {
    Tag,
    FileText
} from 'lucide-react';
import {
    Badge,
    FacetedFilter,
    Tooltip,
    TooltipTrigger,
    TooltipContent,
} from '@ship/primitives';
import { BaseMetadataHeader } from '../common/BaseMetadataHeader';
import { MetadataPopover } from '../common/MetadataPopover';

interface SpecHeaderMetadataProps {
    fileName: string;
    tags?: string[];
    isEditing: boolean;
    onUpdate: (updates: {
        tags?: string[];
    }) => void;
    tagSuggestions?: string[];
}

export function SpecHeaderMetadata({
    fileName,
    tags = [],
    isEditing,
    onUpdate,
    tagSuggestions = [],
}: SpecHeaderMetadataProps) {
    return (
        <BaseMetadataHeader>
            {/* File Name Info */}
            <div className="flex items-center gap-1.5 shrink-0">
                <FileText className="size-3.5" />
                <span className="font-medium text-foreground">{fileName}</span>
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
                <TooltipContent side="bottom">View and manage tags for this spec.</TooltipContent>
            </Tooltip>
        </BaseMetadataHeader>
    );
}
