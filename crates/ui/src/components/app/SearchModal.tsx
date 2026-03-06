import * as React from 'react';
import { useNavigate } from '@tanstack/react-router';
import {
    StickyNote,
    FileText,
    Target,
    Gavel,
    Package,
    Workflow,
    LayoutDashboard,
    Bot,
    Settings,
    FileStack,
    Zap,
    Check,
} from 'lucide-react';
import { useWorkspace, useShip } from '@/lib/hooks/workspace/WorkspaceContext';
import {
    CommandDialog,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem,
    CommandList,
    CommandSeparator,
} from '@ship/ui';
import {
    ADRS_ROUTE,
    FEATURES_ROUTE,
    NOTES_ROUTE,
    OVERVIEW_ROUTE,
    RELEASES_ROUTE,
    WORKFLOW_WORKSPACE_ROUTE,
    SETTINGS_ROUTE,
} from '@/lib/constants/routes';

export function SearchModal() {
    const [open, setOpen] = React.useState(false);
    const navigate = useNavigate();
    const workspace = useWorkspace();
    const ship = useShip();

    const {
        setNotesScope,
    } = workspace;

    const {
        notes,
        specs,
        features,
        adrs,
        releases,
        handleSelectNote,
        handleSelectFeature,
        handleSelectRelease,
        setSelectedSpec,
        setSelectedAdr,
    } = ship;

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
            <CommandInput placeholder="Search or jump to..." />
            <CommandList>
                <CommandEmpty>No results found.</CommandEmpty>

                <CommandGroup heading="Navigation">
                    <CommandItem onSelect={() => runCommand(() => void navigate({ to: WORKFLOW_WORKSPACE_ROUTE }))}>
                        <Workflow className="mr-2 h-4 w-4" />
                        <span>Workspaces</span>
                    </CommandItem>
                    <CommandItem onSelect={() => runCommand(() => void navigate({ to: OVERVIEW_ROUTE }))}>
                        <LayoutDashboard className="mr-2 h-4 w-4" />
                        <span>Overview</span>
                    </CommandItem>
                    <CommandItem onSelect={() => runCommand(() => void navigate({ to: FEATURES_ROUTE }))}>
                        <Target className="mr-2 h-4 w-4" />
                        <span>Features</span>
                    </CommandItem>
                    <CommandItem onSelect={() => runCommand(() => void navigate({ to: RELEASES_ROUTE }))}>
                        <Package className="mr-2 h-4 w-4" />
                        <span>Releases</span>
                    </CommandItem>
                    <CommandItem onSelect={() => runCommand(() => { setNotesScope('project'); void navigate({ to: NOTES_ROUTE }); })}>
                        <StickyNote className="mr-2 h-4 w-4" />
                        <span>Notes</span>
                    </CommandItem>
                    <CommandItem onSelect={() => runCommand(() => void navigate({ to: ADRS_ROUTE }))}>
                        <FileStack className="mr-2 h-4 w-4" />
                        <span>Decisions</span>
                    </CommandItem>
                    <CommandItem onSelect={() => runCommand(() => void navigate({ to: SETTINGS_ROUTE, search: { tab: 'providers' } }))}>
                        <Bot className="mr-2 h-4 w-4" />
                        <span>Agents</span>
                    </CommandItem>
                    <CommandItem onSelect={() => runCommand(() => void navigate({ to: SETTINGS_ROUTE, search: { tab: undefined } }))}>
                        <Settings className="mr-2 h-4 w-4" />
                        <span>Settings</span>
                    </CommandItem>
                </CommandGroup>

                <CommandSeparator />

                {!workspace.noProject && (
                    <CommandGroup heading="Agent Mode">
                        <CommandItem
                            onSelect={() => runCommand(() => workspace.handleSetActiveMode(null))}
                        >
                            <Zap className="mr-2 h-4 w-4" />
                            <span className="flex-1">Default Mode</span>
                            {workspace.activeModeId === null && <Check className="ml-2 h-4 w-4 text-primary" />}
                        </CommandItem>
                        {workspace.modes.map((mode) => (
                            <CommandItem
                                key={mode.id}
                                onSelect={() => runCommand(() => workspace.handleSetActiveMode(mode.id))}
                            >
                                <Zap className="mr-2 h-4 w-4" />
                                <div className="flex-1 min-w-0">
                                    <span>{mode.name}</span>
                                    {mode.description && (
                                        <span className="ml-2 text-xs text-muted-foreground">{mode.description}</span>
                                    )}
                                </div>
                                {workspace.activeModeId === mode.id && <Check className="ml-2 h-4 w-4 text-primary" />}
                            </CommandItem>
                        ))}
                    </CommandGroup>
                )}

                <CommandSeparator />

                {features.length > 0 && (
                    <CommandGroup heading="Features">
                        {features.map((feature) => (
                            <CommandItem
                                key={`feature-${feature.file_name}`}
                                onSelect={() =>
                                    runCommand(() => {
                                        void navigate({ to: FEATURES_ROUTE });
                                        void handleSelectFeature(feature);
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
                                <span>{spec.id}</span>
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
                                        void navigate({ to: RELEASES_ROUTE });
                                        void handleSelectRelease(release);
                                    })
                                }
                            >
                                <Package className="mr-2 h-4 w-4" />
                                <span>{release.version}</span>
                            </CommandItem>
                        ))}
                    </CommandGroup>
                )}

                {notes.length > 0 && (
                    <CommandGroup heading="Notes">
                        {notes.map((note) => (
                            <CommandItem
                                key={`note-${note.id}`}
                                onSelect={() =>
                                    runCommand(() => {
                                        setNotesScope('project');
                                        void navigate({ to: NOTES_ROUTE });
                                        void handleSelectNote(note);
                                    })
                                }
                            >
                                <StickyNote className="mr-2 h-4 w-4" />
                                <span>{note.title || 'Untitled Note'}</span>
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
                                        void navigate({ to: ADRS_ROUTE });
                                        setSelectedAdr(adr);
                                    })
                                }
                            >
                                <Gavel className="mr-2 h-4 w-4" />
                                <span>{adr.adr.metadata.title || adr.file_name}</span>
                            </CommandItem>
                        ))}
                    </CommandGroup>
                )}


            </CommandList>
        </CommandDialog>
    );
}
