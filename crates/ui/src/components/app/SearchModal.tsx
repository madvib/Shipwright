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
    RefreshCw,
    Layers3,
    Shield,
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
    AGENTS_MCP_ROUTE,
    AGENTS_PERMISSIONS_ROUTE,
    AGENTS_PROVIDERS_ROUTE,
    AGENTS_RULES_ROUTE,
    AGENTS_SKILLS_ROUTE,
    FEATURES_ROUTE,
    NOTES_ROUTE,
    OVERVIEW_ROUTE,
    RELEASES_ROUTE,
    WORKFLOW_WORKSPACE_ROUTE,
} from '@/lib/constants/routes';
import {
    activateWorkspaceCmd,
    createWorkspaceCmd,
    getCurrentBranchCmd,
    listWorkspacesCmd,
    syncWorkspaceCmd,
} from '@/lib/platform/tauri/commands';
import { isTauriRuntime } from '@/lib/platform/tauri/runtime';

export function SearchModal() {
    const [open, setOpen] = React.useState(false);
    const [commandBusy, setCommandBusy] = React.useState(false);
    const [commandError, setCommandError] = React.useState<string | null>(null);
    const [runtimeBranch, setRuntimeBranch] = React.useState<string | null>(null);
    const [knownWorkspaceBranches, setKnownWorkspaceBranches] = React.useState<string[]>([]);
    const navigate = useNavigate();
    const workspace = useWorkspace();
    const ship = useShip();

    const {
        setNotesScope,
        setError: setWorkspaceError,
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
        setSelectedAdr,
    } = ship;

    const refreshRuntimeWorkspaceState = React.useCallback(async () => {
        if (!isTauriRuntime() || workspace.noProject) return;

        const branch = await getCurrentBranchCmd();
        setRuntimeBranch(branch);

        const listed = await listWorkspacesCmd();
        if (listed.status === 'ok') {
            setKnownWorkspaceBranches(listed.data.map((entry) => entry.branch));
        }
    }, [workspace.noProject]);

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

    React.useEffect(() => {
        if (!open) return;
        void refreshRuntimeWorkspaceState();
    }, [open, refreshRuntimeWorkspaceState]);

    const runCommand = React.useCallback(async (command: () => void | Promise<void>) => {
        setOpen(false);
        setCommandError(null);
        try {
            await command();
        } catch (error) {
            setCommandError(String(error));
        }
    }, []);

    const runWorkspaceMutation = React.useCallback(async (mutation: () => Promise<void>) => {
        if (!isTauriRuntime()) return;

        setCommandBusy(true);
        setCommandError(null);
        try {
            await mutation();
            await workspace.refreshActivity();
            await ship.loadShipData();
            await refreshRuntimeWorkspaceState();
        } catch (error) {
            setCommandError(String(error));
        } finally {
            setCommandBusy(false);
        }
    }, [refreshRuntimeWorkspaceState, ship, workspace]);

    const syncCurrentWorkspace = React.useCallback(async () => {
        if (!runtimeBranch) return;
        await runWorkspaceMutation(async () => {
            const result = await syncWorkspaceCmd(runtimeBranch);
            if (result.status === 'error') {
                throw new Error(result.error || `Failed to sync workspace for ${runtimeBranch}`);
            }
        });
    }, [runWorkspaceMutation, runtimeBranch]);

    const ensureCurrentWorkspace = React.useCallback(async () => {
        if (!runtimeBranch) return;
        await runWorkspaceMutation(async () => {
            const workspaceExists = knownWorkspaceBranches.includes(runtimeBranch);
            const result = workspaceExists
                ? await activateWorkspaceCmd(runtimeBranch)
                : await createWorkspaceCmd(runtimeBranch, { activate: true });
            if (result.status === 'error') {
                throw new Error(result.error || `Failed to ensure workspace for ${runtimeBranch}`);
            }
        });
    }, [knownWorkspaceBranches, runWorkspaceMutation, runtimeBranch]);

    const openSpecContext = React.useCallback(async (spec: typeof specs[number]) => {
        const relatedFeature = features.find(
            (feature) => feature.spec_id === spec.id || feature.spec_id === spec.file_name
        );
        if (relatedFeature) {
            await navigate({ to: FEATURES_ROUTE });
            await handleSelectFeature(relatedFeature);
            return;
        }

        await navigate({ to: FEATURES_ROUTE });
        await ship.handleSelectSpec(spec);
        const message = `Spec "${spec.spec.metadata.title || spec.id}" is not linked to a feature yet.`;
        setWorkspaceError(message);
        throw new Error(message);
    }, [features, handleSelectFeature, navigate, setWorkspaceError, ship]);

    const openSettingsSection = React.useCallback((section: 'providers' | 'mcp' | 'skills' | 'rules' | 'permissions') => {
        const routeBySection = {
            providers: AGENTS_PROVIDERS_ROUTE,
            mcp: AGENTS_MCP_ROUTE,
            skills: AGENTS_SKILLS_ROUTE,
            rules: AGENTS_RULES_ROUTE,
            permissions: AGENTS_PERMISSIONS_ROUTE,
        } as const;
        void navigate({ to: routeBySection[section] });
    }, [navigate]);

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
                    <CommandItem onSelect={() => runCommand(() => openSettingsSection('providers'))}>
                        <Bot className="mr-2 h-4 w-4" />
                        <span>Agents</span>
                    </CommandItem>
                    <CommandItem onSelect={() => runCommand(() => openSettingsSection('providers'))}>
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

                {!workspace.noProject && (
                    <CommandGroup heading="Workflow Control">
                        <CommandItem
                            disabled={!runtimeBranch || commandBusy}
                            onSelect={() => void runCommand(syncCurrentWorkspace)}
                        >
                            <RefreshCw className="mr-2 h-4 w-4" />
                            <span className="flex-1">
                                {runtimeBranch ? `Sync Workspace (${runtimeBranch})` : 'Sync Current Workspace'}
                            </span>
                        </CommandItem>
                        <CommandItem
                            disabled={!runtimeBranch || commandBusy}
                            onSelect={() => void runCommand(ensureCurrentWorkspace)}
                        >
                            <Workflow className="mr-2 h-4 w-4" />
                            <span className="flex-1">
                                {runtimeBranch
                                    ? `Create/Activate Workspace (${runtimeBranch})`
                                    : 'Create/Activate Current Branch Workspace'}
                            </span>
                        </CommandItem>
                        <CommandItem onSelect={() => void runCommand(async () => { await workspace.refreshActivity(); await ship.loadShipData(); })}>
                            <RefreshCw className="mr-2 h-4 w-4" />
                            <span className="flex-1">Refresh Project Context</span>
                        </CommandItem>
                    </CommandGroup>
                )}

                <CommandSeparator />

                <CommandGroup heading="Agent Control Plane">
                    <CommandItem onSelect={() => void runCommand(() => openSettingsSection('providers'))}>
                        <Bot className="mr-2 h-4 w-4" />
                        <span>Providers</span>
                    </CommandItem>
                    <CommandItem onSelect={() => void runCommand(() => openSettingsSection('mcp'))}>
                        <Layers3 className="mr-2 h-4 w-4" />
                        <span>MCP Servers</span>
                    </CommandItem>
                    <CommandItem onSelect={() => void runCommand(() => openSettingsSection('skills'))}>
                        <FileStack className="mr-2 h-4 w-4" />
                        <span>Skills</span>
                    </CommandItem>
                    <CommandItem onSelect={() => void runCommand(() => openSettingsSection('rules'))}>
                        <Gavel className="mr-2 h-4 w-4" />
                        <span>Rules</span>
                    </CommandItem>
                    <CommandItem onSelect={() => void runCommand(() => openSettingsSection('permissions'))}>
                        <Shield className="mr-2 h-4 w-4" />
                        <span>Permissions</span>
                    </CommandItem>
                </CommandGroup>

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
                                    runCommand(() => openSpecContext(spec))
                                }
                            >
                                <FileText className="mr-2 h-4 w-4" />
                                <div className="flex min-w-0 flex-1 items-center gap-2">
                                    <span className="truncate">
                                        {spec.spec.metadata.title || spec.id}
                                    </span>
                                    <span className="shrink-0 text-[10px] text-muted-foreground">
                                        {spec.id}
                                    </span>
                                </div>
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

                {commandError && (
                    <>
                        <CommandSeparator />
                        <CommandGroup heading="Command Error">
                            <CommandItem disabled>
                                <span className="text-xs text-destructive truncate">{commandError}</span>
                            </CommandItem>
                        </CommandGroup>
                    </>
                )}

            </CommandList>
        </CommandDialog>
    );
}
