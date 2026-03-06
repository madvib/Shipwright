import {
    Tag,
    FileText
} from 'lucide-react';
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
    Button,
    Badge,
    FacetedFilter,
} from '@ship/ui';

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
        <div className="flex flex-nowrap items-center gap-4 text-xs text-muted-foreground overflow-hidden">
            {/* File Name Info */}
            <div className="flex items-center gap-1.5 shrink-0">
                <FileText className="size-3.5" />
                <span className="font-medium text-foreground">{fileName}</span>
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
