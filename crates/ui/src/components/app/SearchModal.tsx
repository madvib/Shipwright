import * as React from 'react';
import { useNavigate } from '@tanstack/react-router';
import {
    StickyNote,
    FileText,
    Target,
    Gavel,
    CheckCircle2,
    Package,
} from 'lucide-react';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';
import {
    CommandDialog,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem,
    CommandList,
} from '@/components/ui/command';
import { NOTES_ROUTE } from '@/lib/constants/routes';

export function SearchModal() {
    const [open, setOpen] = React.useState(false);
    const navigate = useNavigate();
    const {
        notes,
        specs,
        features,
        adrs,
        issues,
        releases,
        setSelectedNote,
        setSelectedSpec,
        setSelectedFeature,
        setSelectedAdr,
        setSelectedIssue,
        setSelectedRelease,
    } = useWorkspace();

    React.useEffect(() => {
        const down = (e: KeyboardEvent) => {
            if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
                e.preventDefault();
                setOpen((open) => !open);
            }
        };

        document.addEventListener('keydown', down);
        return () => document.removeEventListener('keydown', down);
    }, []);

    const runCommand = React.useCallback((command: () => void) => {
        setOpen(false);
        command();
    }, []);

    return (
        <CommandDialog open={open} onOpenChange={setOpen}>
            <CommandInput placeholder="Search project..." />
            <CommandList>
                <CommandEmpty>No results found.</CommandEmpty>

                {notes.length > 0 && (
                    <CommandGroup heading="Notes">
                        {notes.map((note) => (
                            <CommandItem
                                key={`note-${note.file_name}`}
                                onSelect={() =>
                                    runCommand(() => {
                                        void navigate({ to: NOTES_ROUTE });
                                        setSelectedNote(note as any); // Cast as it will fetch the full doc on select in the context
                                    })
                                }
                            >
                                <StickyNote className="mr-2 h-4 w-4" />
                                <span>{note.title || 'Untitled Note'}</span>
                            </CommandItem>
                        ))}
                    </CommandGroup>
                )}

                {features.length > 0 && (
                    <CommandGroup heading="Features">
                        {features.map((feature) => (
                            <CommandItem
                                key={`feature-${feature.file_name}`}
                                onSelect={() =>
                                    runCommand(() => {
                                        setSelectedFeature(feature as any);
                                    })
                                }
                            >
                                <Target className="mr-2 h-4 w-4" />
                                <span>{feature.title}</span>
                            </CommandItem>
                        ))}
                    </CommandGroup>
                )}

                {specs.length > 0 && (
                    <CommandGroup heading="Specs">
                        {specs.map((spec) => (
                            <CommandItem
                                key={`spec-${spec.file_name}`}
                                onSelect={() =>
                                    runCommand(() => {
                                        setSelectedSpec(spec as any);
                                    })
                                }
                            >
                                <FileText className="mr-2 h-4 w-4" />
                                <span>{spec.title}</span>
                            </CommandItem>
                        ))}
                    </CommandGroup>
                )}

                {adrs.length > 0 && (
                    <CommandGroup heading="Decisions (ADRs)">
                        {adrs.map((adr) => (
                            <CommandItem
                                key={`adr-${adr.file_name}`}
                                onSelect={() =>
                                    runCommand(() => {
                                        setSelectedAdr(adr);
                                    })
                                }
                            >
                                <Gavel className="mr-2 h-4 w-4" />
                                <span>{adr.adr.metadata.title}</span>
                            </CommandItem>
                        ))}
                    </CommandGroup>
                )}

                {issues.length > 0 && (
                    <CommandGroup heading="Issues">
                        {issues.map((issue) => (
                            <CommandItem
                                key={`issue-${issue.file_name}`}
                                onSelect={() =>
                                    runCommand(() => {
                                        setSelectedIssue(issue);
                                    })
                                }
                            >
                                <CheckCircle2 className="mr-2 h-4 w-4" />
                                <span>{issue.issue.title}</span>
                            </CommandItem>
                        ))}
                    </CommandGroup>
                )}

                {releases.length > 0 && (
                    <CommandGroup heading="Releases">
                        {releases.map((release) => (
                            <CommandItem
                                key={`release-${release.file_name}`}
                                onSelect={() =>
                                    runCommand(() => {
                                        setSelectedRelease(release as any);
                                    })
                                }
                            >
                                <Package className="mr-2 h-4 w-4" />
                                <span>{release.version}</span>
                            </CommandItem>
                        ))}
                    </CommandGroup>
                )}
            </CommandList>
        </CommandDialog>
    );
}
