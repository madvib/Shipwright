import { createRouter } from '@tanstack/react-router';
import { rootRoute } from './routes/__root';
import { indexRoute } from './routes/index';
import { overviewRoute } from './routes/overview';
import { projectsRoute } from './routes/projects';
import { issuesRoute } from './routes/issues';
import { specsRoute } from './routes/specs';
import { adrsRoute } from './routes/adrs';
import { activityRoute } from './routes/activity';
import { agentsRoute } from './routes/agents';
import { settingsRoute } from './routes/settings';

const routeTree = rootRoute.addChildren([
  indexRoute,
  overviewRoute,
  projectsRoute,
  issuesRoute,
  specsRoute,
  adrsRoute,
  activityRoute,
  agentsRoute,
  settingsRoute,
]);

export const router = createRouter({
  routeTree,
  defaultPreload: 'intent',
});

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
