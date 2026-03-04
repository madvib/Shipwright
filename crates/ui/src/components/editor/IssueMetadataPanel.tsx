import AutocompleteInput from '@ship/ui';
import { FacetedFilter } from '@ship/ui';
import { FieldLabel } from '@ship/ui';
import { Input } from '@ship/ui';
import { StatusConfig } from '@/bindings';
import { getStatusStyles } from '@/lib/workspace-ui';
import { cn } from '@/lib/utils';

interface IssueMetadataPanelProps {
    title: string;
    assignee: string | null;
    specId: string | null;
    tags: string[];
    status: string;
    statuses: StatusConfig[];
    tagSuggestions: string[];
    specSuggestions: string[];
    onTitleChange: (v: string) => void;
    onAssigneeChange: (v: string | null) => void;
    onSpecIdChange: (v: string | null) => void;
    onTagsChange: (v: string[]) => void;
    onStatusChange: (v: string) => void;
}

export default function IssueMetadataPanel({
    title,
    assignee,
    specId,
    tags,
    status,
    statuses,
    tagSuggestions,
    specSuggestions,
    onTitleChange,
    onAssigneeChange,
    onSpecIdChange,
    onTagsChange,
    onStatusChange,
}: IssueMetadataPanelProps) {
    return (
        <aside className="flex h-full min-h-0 flex-col overflow-y-auto border-l bg-muted/20">
            {/* Properties header */}
            <div className="flex items-center gap-2 border-b bg-card/50 px-4 py-3">
                <p className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground">Properties</p>
            </div>

            <div className="flex flex-col gap-4 p-4">
                {/* Status */}
                <div className="space-y-1.5">
                    <FieldLabel>Status</FieldLabel>
                    <div className="flex flex-wrap gap-1.5">
                        {statuses.map((s) => {
                            const style = getStatusStyles(s);
                            const active = status === s.id;
                            return (
                                <button
                                    key={s.id}
                                    type="button"
                                    onClick={() => onStatusChange(s.id)}
                                    className={cn(
                                        'rounded-md border px-2.5 py-1 text-xs font-medium transition-all',
                                        active
                                            ? `${style.bg} ${style.color} ${style.border} shadow-sm`
                                            : 'border-border/50 hover:bg-muted/80 text-muted-foreground'
                                    )}
                                >
                                    {style.label}
                                </button>
                            );
                        })}
                    </div>
                </div>

                {/* Title */}
                <div className="space-y-1.5">
                    <FieldLabel>Title</FieldLabel>
                    <Input
                        value={title}
                        className="h-8"
                        placeholder="Issue title"
                        onChange={(e) => onTitleChange(e.target.value)}
                    />
                </div>

                {/* Assignee */}
                <div className="space-y-1.5">
                    <FieldLabel>Assignee</FieldLabel>
                    <Input
                        value={assignee ?? ''}
                        className="h-8"
                        placeholder="Unassigned"
                        onChange={(e) => onAssigneeChange(e.target.value.trim() || null)}
                    />
                </div>

                {/* Spec */}
                <div className="space-y-1.5">
                    <FieldLabel>Spec</FieldLabel>
                    <AutocompleteInput
                        value={specId ?? ''}
                        options={specSuggestions.map((s) => ({ value: s }))}
                        className="h-8"
                        placeholder="Link a spec"
                        noResultsText="No specs found."
                        onValueChange={(v) => onSpecIdChange(v.trim() || null)}
                    />
                </div>

                {/* Tags */}
                <div className="space-y-1.5">
                    <FieldLabel>Tags {tags.length > 0 && `(${tags.length})`}</FieldLabel>
                    <FacetedFilter
                        title="Add tag"
                        options={tagSuggestions.map((t) => ({ label: t, value: t }))}
                        selectedValues={tags}
                        onSelectionChange={onTagsChange}
                        allowNew
                        onAddNew={(tag) => onTagsChange([...tags, tag])}
                    />
                </div>
            </div>
        </aside>
    );
}
