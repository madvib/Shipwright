import { Badge, Button, Select, SelectContent, SelectItem, SelectTrigger, FacetedFilter, Popover, PopoverContent, PopoverTrigger } from '@ship/ui';
import { User, Tag, FileCode2 } from 'lucide-react';
import { StatusConfig } from '@/bindings';
import { cn } from '@/lib/utils';

interface IssueHeaderMetadataProps {
    status: string;
    statuses: StatusConfig[];
    assignee: string | null;
    specId: string | null;
    tags: string[];
    tagSuggestions: string[];
    specSuggestions: string[];
    onStatusChange: (status: string) => void;
    onAssigneeChange: (assignee: string | null) => void;
    onSpecIdChange: (specId: string | null) => void;
    onTagsChange: (tags: string[]) => void;
}

export function IssueHeaderMetadata({
    status,
    statuses,
    assignee,
    specId,
    tags,
    tagSuggestions,
    specSuggestions,
    onStatusChange,
    onAssigneeChange,
    onSpecIdChange,
    onTagsChange,
}: IssueHeaderMetadataProps) {
    const activeStatus = statuses.find(s => s.id === status) || statuses[0];

    return (
        <div className="flex flex-wrap items-center gap-2 py-1">
            {/* Status */}
            <Select value={status} onValueChange={(next) => next && onStatusChange(next)}>
                <SelectTrigger className="h-7 border-none bg-muted/50 px-2 py-0 text-xs font-medium hover:bg-muted focus:ring-0">
                    <Badge variant="outline" className="border-none bg-transparent p-0 text-xs shadow-none">
                        {activeStatus?.name || status}
                    </Badge>
                </SelectTrigger>
                <SelectContent>
                    {statuses.map((s) => (
                        <SelectItem key={s.id} value={s.id}>
                            {s.name}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>

            {/* Assignee */}
            <Select value={assignee || "unassigned"} onValueChange={(v) => onAssigneeChange(v === "unassigned" ? null : v)}>
                <SelectTrigger className="h-7 border-none bg-muted/50 px-2 py-0 text-xs hover:bg-muted focus:ring-0">
                    <div className="flex items-center gap-1.5">
                        <User className="size-3 text-muted-foreground" />
                        <span className={cn(!assignee && "text-muted-foreground")}>
                            {assignee || "Unassigned"}
                        </span>
                    </div>
                </SelectTrigger>
                <SelectContent>
                    <SelectItem value="unassigned">Unassigned</SelectItem>
                    {assignee && <SelectItem key={assignee} value={assignee}>{assignee}</SelectItem>}
                </SelectContent>
            </Select>

            {/* Linked Spec */}
            <Select value={specId || "none"} onValueChange={(v) => onSpecIdChange(v === "none" ? null : v)}>
                <SelectTrigger className="h-7 border-none bg-muted/50 px-2 py-0 text-xs hover:bg-muted focus:ring-0 max-w-[150px]">
                    <div className="flex items-center gap-1.5 truncate">
                        <FileCode2 className="size-3 text-muted-foreground" />
                        <span className={cn(!specId && "text-muted-foreground", "truncate")}>
                            {specId || "No spec"}
                        </span>
                    </div>
                </SelectTrigger>
                <SelectContent>
                    <SelectItem value="none">No spec</SelectItem>
                    {specSuggestions.map(id => (
                        <SelectItem key={id} value={id}>{id}</SelectItem>
                    ))}
                </SelectContent>
            </Select>

            {/* Tags Popover */}
            <Popover>
                <PopoverTrigger
                    render={
                        <Button
                            variant="ghost"
                            size="xs"
                            className="h-7 gap-1.5 px-2 text-xs font-normal text-muted-foreground hover:bg-muted focus:ring-0"
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
                        <FacetedFilter
                            title="Edit Tags"
                            options={tagSuggestions.map(t => ({ label: t, value: t }))}
                            selectedValues={tags}
                            onSelectionChange={onTagsChange}
                            allowNew
                            onAddNew={(tag: string) => onTagsChange([...tags, tag])}
                        />
                    </div>
                </PopoverContent>
            </Popover>
        </div>
    );
}
