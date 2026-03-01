import { useState, useEffect } from 'react';
import { GitBranch, Box, Loader2, Info, Zap, RefreshCw } from 'lucide-react';
import { getCurrentBranchCmd, getWorkspaceCmd } from '@/lib/platform/tauri/commands';
import { Workspace } from '@/bindings';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { PageFrame, PageHeader } from '@/components/app/PageFrame';

export default function WorkspacePanel() {
    const [branch, setBranch] = useState<string | null>(null);
    const [workspace, setWorkspace] = useState<Workspace | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const load = async () => {
        setLoading(true);
        setError(null);
        try {
            const currentBranch = await getCurrentBranchCmd();
            setBranch(currentBranch);

            if (currentBranch) {
                const result = await getWorkspaceCmd(currentBranch);
                if (result.status === 'ok') {
                    setWorkspace(result.data);
                } else {
                    setError(result.error || 'Failed to load workspace.');
                }
            } else {
                setWorkspace(null);
            }
        } catch (err) {
            setError(String(err));
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        void load();
    }, []);

    const actions = (
        <Button variant="outline" size="xs" onClick={() => void load()} disabled={loading}>
            <RefreshCw className={`size-3.5 ${loading ? 'animate-spin' : ''}`} />
            Refresh
        </Button>
    );

    if (loading) {
        return (
            <PageFrame>
                <div className="flex h-64 items-center justify-center">
                    <Loader2 className="size-6 animate-spin text-muted-foreground" />
                </div>
            </PageFrame>
        );
    }

    if (error) {
        return (
            <PageFrame>
                <Alert variant="destructive">
                    <AlertTitle>Error</AlertTitle>
                    <AlertDescription>{error}</AlertDescription>
                </Alert>
            </PageFrame>
        );
    }

    if (!branch) {
        return (
            <PageFrame>
                <PageHeader
                    eyebrow="Workflow"
                    title="Workspace"
                    description="Active development context for the current branch."
                    actions={actions}
                />
                <div className="flex flex-col items-center justify-center gap-4 rounded-xl border border-dashed bg-muted/20 py-16 text-center">
                    <Box className="size-12 text-muted-foreground/30" />
                    <div>
                        <h3 className="text-base font-semibold">No Git Repository Found</h3>
                        <p className="mt-1 max-w-md text-sm text-muted-foreground">
                            The active project doesn't appear to be inside a git repository. Workspace context requires git.
                        </p>
                    </div>
                </div>
            </PageFrame>
        );
    }

    if (!workspace) {
        return (
            <PageFrame>
                <PageHeader
                    eyebrow="Workflow"
                    title="Workspace"
                    description="Active development context for the current branch."
                    actions={actions}
                />
                <div className="flex flex-col items-center justify-center gap-4 rounded-xl border border-dashed bg-muted/20 py-16 text-center">
                    <GitBranch className="size-12 text-muted-foreground/30" />
                    <div>
                        <h3 className="text-base font-semibold">No Workspace Session</h3>
                        <p className="mt-1 max-w-md text-sm text-muted-foreground">
                            No workspace session found for branch{' '}
                            <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{branch}</code>.
                            Workspace sessions are created automatically when you switch branches via the git post-checkout hook.
                        </p>
                    </div>
                </div>
            </PageFrame>
        );
    }

    return (
        <PageFrame>
            <PageHeader
                eyebrow="Workflow"
                title="Workspace"
                description="Active development context for the current branch."
                actions={actions}
            />

            <div className="grid gap-4 md:grid-cols-2">
                {/* Branch */}
                <Card className="border-primary/20 bg-gradient-to-br from-primary/5 to-transparent shadow-sm">
                    <CardHeader className="pb-2">
                        <div className="flex items-center gap-2">
                            <div className="flex size-8 items-center justify-center rounded-lg bg-primary/10">
                                <GitBranch className="size-4 text-primary" />
                            </div>
                            <div>
                                <CardTitle className="text-sm">Active Branch</CardTitle>
                                <CardDescription className="text-[11px]">Current git branch being tracked</CardDescription>
                            </div>
                        </div>
                    </CardHeader>
                    <CardContent>
                        <div className="flex items-center gap-2">
                            <code className="rounded-lg border bg-background px-2.5 py-1 font-mono text-sm font-semibold text-primary">
                                {workspace.branch}
                            </code>
                            {workspace.is_worktree && (
                                <Badge variant="secondary" className="text-[10px] uppercase tracking-wider">Worktree</Badge>
                            )}
                        </div>
                    </CardContent>
                </Card>

                {/* Context */}
                <Card className="border-accent/20 bg-gradient-to-br from-accent/5 to-transparent shadow-sm">
                    <CardHeader className="pb-2">
                        <div className="flex items-center gap-2">
                            <div className="flex size-8 items-center justify-center rounded-lg bg-accent/10">
                                <Zap className="size-4 text-accent" />
                            </div>
                            <div>
                                <CardTitle className="text-sm">Target Context</CardTitle>
                                <CardDescription className="text-[11px]">Feature and spec linked to this session</CardDescription>
                            </div>
                        </div>
                    </CardHeader>
                    <CardContent className="space-y-2">
                        <div className="flex items-center justify-between gap-2">
                            <span className="text-xs text-muted-foreground font-medium uppercase tracking-wider">Feature</span>
                            {workspace.feature_id ? (
                                <code className="rounded border bg-muted/50 px-1.5 py-0.5 font-mono text-xs truncate max-w-[180px]">{workspace.feature_id}</code>
                            ) : (
                                <span className="text-xs text-muted-foreground italic">None</span>
                            )}
                        </div>
                        <div className="flex items-center justify-between gap-2">
                            <span className="text-xs text-muted-foreground font-medium uppercase tracking-wider">Spec</span>
                            {workspace.spec_id ? (
                                <code className="rounded border bg-muted/50 px-1.5 py-0.5 font-mono text-xs truncate max-w-[180px]">{workspace.spec_id}</code>
                            ) : (
                                <span className="text-xs text-muted-foreground italic">None</span>
                            )}
                        </div>
                    </CardContent>
                </Card>

                {/* Worktree path */}
                {workspace.worktree_path && (
                    <Card className="border-border/40 bg-muted/10 shadow-none md:col-span-2">
                        <CardHeader className="pb-2">
                            <CardTitle className="text-[10px] font-black uppercase tracking-widest text-muted-foreground">Worktree Path</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <code className="block truncate rounded-lg border border-border/30 bg-background/50 p-2 font-mono text-xs">
                                {workspace.worktree_path}
                            </code>
                        </CardContent>
                    </Card>
                )}

                {/* Session info */}
                <Alert className="border-primary/20 bg-primary/5 md:col-span-2">
                    <Info className="size-4 text-primary" />
                    <AlertTitle className="text-xs font-semibold uppercase tracking-wider text-primary">Session Info</AlertTitle>
                    <AlertDescription className="text-xs text-muted-foreground">
                        Workspace resolved at{' '}
                        <span className="font-mono">{new Date(workspace.resolved_at).toLocaleString()}</span>.
                        All changes on this branch are tracked as part of the current development loop.
                    </AlertDescription>
                </Alert>
            </div>
        </PageFrame>
    );
}
