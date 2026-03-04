import { createFileRoute } from '@tanstack/react-router';
import IssueList from '@/features/planning/IssueList';
import { Badge } from '@ship/ui';
import { Button } from '@ship/ui';
import { EmptyState } from '@ship/ui';
import { PageFrame, PageHeader } from '@/components/app/PageFrame';
import TemplateEditorButton from '@/features/planning/TemplateEditorButton';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';
import { Plus } from 'lucide-react';

function IssuesRouteComponent() {
  const workspace = useWorkspace();
  const totalIssues = workspace.issues.length;
  const statusTotals = workspace.statuses.map((status) => ({
    id: status.id,
    label: status.name,
    count: workspace.issues.filter((issue) => issue.status === status.id).length,
  }));

  return (
    <PageFrame width="wide" className="overflow-x-hidden">
      <PageHeader
        title="Issues"
        description={`${totalIssues} issue${totalIssues !== 1 ? 's' : ''} across ${workspace.statuses.length} categories`}
        actions={
          <div className="flex items-center gap-2">
            <TemplateEditorButton kind="issue" />
            <Button onClick={() => workspace.setShowNewIssue(true)}>+ New Issue</Button>
          </div>
        }
        footer={
          statusTotals.length > 0 ? (
            <div className="flex flex-wrap gap-2">
              {statusTotals.map((status) => (
                <Badge
                  key={status.id}
                  variant={workspace.statuses.find((s) => s.id === status.id)?.color as any}
                >
                  {status.label}: {status.count}
                </Badge>
              ))}
            </div>
          ) : null
        }
      />
      {totalIssues === 0 ? (
        <EmptyState
          title="No issues yet"
          description="Create your first issue to start tracking work."
          action={
            <Button onClick={() => workspace.setShowNewIssue(true)}>
              <Plus className="mr-2 size-4" />
              Create First Issue
            </Button>
          }
        />
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
