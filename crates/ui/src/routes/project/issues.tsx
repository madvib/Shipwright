import { createFileRoute } from '@tanstack/react-router';
import IssueList from '@/features/planning/IssueList';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { PageFrame, PageHeader } from '@/components/app/PageFrame';
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
    <PageFrame width="wide" className="overflow-x-hidden">
      <PageHeader
        title="Issues"
        description={`${totalIssues} issue${totalIssues !== 1 ? 's' : ''} across ${workspace.statuses.length} categories`}
        actions={<Button onClick={() => workspace.setShowNewIssue(true)}>+ New Issue</Button>}
        footer={
          statusTotals.length > 0 ? (
            <div className="flex flex-wrap gap-2">
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
          ) : null
        }
      />
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
    </PageFrame>
  );
}

export const Route = createFileRoute('/project/issues')({
  component: IssuesRouteComponent,
});
