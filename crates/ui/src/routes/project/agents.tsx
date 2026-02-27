import { Outlet, createFileRoute, redirect } from '@tanstack/react-router';
import { AGENTS_PROVIDERS_ROUTE, AGENTS_ROUTE } from '@/lib/constants/routes';

export const Route = createFileRoute('/project/agents')({
  beforeLoad: ({ location }) => {
    if (location.pathname === AGENTS_ROUTE) {
      throw redirect({ to: AGENTS_PROVIDERS_ROUTE });
    }
  },
  component: Outlet,
});
