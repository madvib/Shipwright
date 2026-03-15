import { Outlet, createFileRoute, redirect } from '@tanstack/react-router';
import { SETTINGS_ROUTE } from '@/lib/constants/routes';

export const Route = createFileRoute('/project/agents')({
  beforeLoad: () => {
    throw redirect({ to: SETTINGS_ROUTE, search: { tab: 'providers' } });
  },
  component: Outlet,
});
