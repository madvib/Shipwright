import { createFileRoute, redirect } from '@tanstack/react-router';
import { SETTINGS_ROUTE } from '@/lib/constants/routes';

export const Route = createFileRoute('/project/agents/rules')({
  beforeLoad: () => {
    throw redirect({ to: SETTINGS_ROUTE, search: { tab: 'rules' } });
  },
  component: () => null,
});
