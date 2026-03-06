import { Navigate, createFileRoute } from '@tanstack/react-router';
import { WORKFLOW_WORKSPACE_ROUTE } from '@/lib/constants/routes';

export const Route = createFileRoute('/')({
  component: () => <Navigate to={WORKFLOW_WORKSPACE_ROUTE} />,
});
