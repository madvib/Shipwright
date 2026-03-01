import { createFileRoute } from '@tanstack/react-router';
import WorkspacePanel from '@/features/workflow/WorkspacePanel';

export const Route = createFileRoute('/project/workflow/workspace')({
    component: WorkspacePanel,
});
