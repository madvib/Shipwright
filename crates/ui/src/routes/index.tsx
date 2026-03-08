import { Navigate, createFileRoute } from '@tanstack/react-router';
import { OVERVIEW_ROUTE } from '@/lib/constants/routes';

export const Route = createFileRoute('/')({
  component: () => <Navigate to={OVERVIEW_ROUTE} />,
});
