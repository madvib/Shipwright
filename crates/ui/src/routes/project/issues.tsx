import { createFileRoute } from '@tanstack/react-router';
import IssueList from '@/features/planning/IssueList';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';
import { getStatusStyles } from '@/lib/workspace-ui';

function IssuesRouteComponent() {
  const workspace = useWorkspace();
  const totalIssues = workspace.issues.length;
  const statusTotals = workspace.statuses.map((status) => ({
    id: status.id,
    label: status.name,
    styles: getStatusStyles(status),
    count: workspace.issues.filter((issue) => issue.status === status.id).length,
  }));

  return (
    <div className="mx-auto flex w-full max-w-[1550px] flex-col gap-4 overflow-x-hidden p-5 md:p-6">
      <header className="relative overflow-hidden rounded-xl border border-primary/25 bg-gradient-to-br from-primary/18 via-accent/10 to-transparent p-4 md:p-5">
        <div className="pointer-events-none absolute -right-24 -top-24 size-56 rounded-full bg-accent/20 blur-3xl" />
        <div className="relative flex flex-wrap items-start justify-between gap-3">
          <div>
            <h1 className="text-2xl font-semibold tracking-tight">{workspace.activeProject?.name ?? 'Project'}</h1>
            <p className="text-muted-foreground text-sm">
              {totalIssues} issue{totalIssues !== 1 ? 's' : ''} across {workspace.statuses.length} categories
            </p>
          </div>
          <Button onClick={() => workspace.setShowNewIssue(true)}>+ New Issue</Button>
        </div>

        {statusTotals.length > 0 && (
          <div className="relative mt-3 flex flex-wrap gap-2">
            {statusTotals.map((status) => (
              <Badge
                key={status.id}
                variant="outline"
                className={`${status.styles.border} ${status.styles.bg} ${status.styles.color}`}
              >
                {status.label}: {status.count}
              </Badge>
            ))}
          </div>
        )}
      </header>
      {totalIssues === 0 ? (
        <Card size="sm">
          <CardHeader>
            <CardTitle>No issues yet</CardTitle>
            <CardDescription>Create your first issue to start tracking work.</CardDescription>
          </CardHeader>
          <CardContent>
            <Button onClick={() => workspace.setShowNewIssue(true)}>Create First Issue</Button>
          </CardContent>
        </Card>
      ) : (
        <IssueList
          issues={workspace.issues}
          statuses={workspace.statuses}
          onSelect={workspace.setSelectedIssue}
          onMove={(fileName, fromStatus, toStatus) =>
            workspace.handleStatusChange(fileName, fromStatus, toStatus, { selectMovedIssue: false })
          }
          onNewIssue={() => workspace.setShowNewIssue(true)}
        />
      )}
    </div>
  );
}

export const Route = createFileRoute('/project/issues')({
  component: IssuesRouteComponent,
});
