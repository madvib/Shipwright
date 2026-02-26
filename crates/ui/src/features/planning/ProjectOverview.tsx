import {
  ArrowRight,
  FileCode2,
  FileStack,
  Flag,
  FolderGit2,
  Folders,
  Package,
  ScrollText,
} from 'lucide-react';
import {
  AdrEntry,
  EventRecord,
  FeatureInfo as FeatureEntry,
  IssueEntry,
  ProjectDiscovery as Project,
  ReleaseInfo as ReleaseEntry,
  SpecInfo as SpecEntry,
  StatusConfig,
} from '@/bindings';
import { getStatusStyles } from '@/lib/workspace-ui';
import {
  ACTIVITY_ROUTE,
  ADRS_ROUTE,
  AppRoutePath,
  FEATURES_ROUTE,
  ISSUES_ROUTE,
  PROJECTS_ROUTE,
  RELEASES_ROUTE,
  SPECS_ROUTE,
} from '@/lib/constants/routes';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

interface ProjectOverviewProps {
  project: Project;
  issues: IssueEntry[];
  specs: SpecEntry[];
  adrs: AdrEntry[];
  releases: ReleaseEntry[];
  features: FeatureEntry[];
  events: EventRecord[];
  statuses: StatusConfig[];
  onNavigate: (path: AppRoutePath) => void;
}

export default function ProjectOverview({
  project,
  issues,
  specs,
  adrs,
  releases,
  features,
  events,
  statuses,
  onNavigate,
}: ProjectOverviewProps) {
  const doneStatusIds = statuses
    .filter((status) => /done|closed|complete/i.test(status.id))
    .map((status) => status.id);
  const openIssues = issues.filter((entry) => !doneStatusIds.includes(entry.status));

  return (
    <div className="mx-auto flex w-full max-w-6xl flex-col gap-4 p-5 md:p-6">
      <header className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">{project.name}</h1>
          <p className="text-muted-foreground text-sm">{project.path}</p>
        </div>
        <Button variant="outline" onClick={() => onNavigate(PROJECTS_ROUTE)}>
          <Folders className="size-4" />
          Switch Project
        </Button>
      </header>

      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <Card size="sm">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Open Issues</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <p className="text-2xl font-semibold">{openIssues.length}</p>
            <Button size="sm" variant="outline" onClick={() => onNavigate(ISSUES_ROUTE)}>
              Open Board
              <ArrowRight className="size-3.5" />
            </Button>
          </CardContent>
        </Card>
        <Card size="sm">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Specs</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <p className="text-2xl font-semibold">{specs.length}</p>
            <Button size="sm" variant="outline" onClick={() => onNavigate(SPECS_ROUTE)}>
              <FileCode2 className="size-4" />
              Open Specs
            </Button>
          </CardContent>
        </Card>
        <Card size="sm">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">ADRs</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <p className="text-2xl font-semibold">{adrs.length}</p>
            <Button size="sm" variant="outline" onClick={() => onNavigate(ADRS_ROUTE)}>
              <FileStack className="size-4" />
              Open ADRs
            </Button>
          </CardContent>
        </Card>
        <Card size="sm">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Releases</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <p className="text-2xl font-semibold">{releases.length}</p>
            <Button size="sm" variant="outline" onClick={() => onNavigate(RELEASES_ROUTE)}>
              <Package className="size-4" />
              Open Releases
            </Button>
          </CardContent>
        </Card>
        <Card size="sm">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Features</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <p className="text-2xl font-semibold">{features.length}</p>
            <Button size="sm" variant="outline" onClick={() => onNavigate(FEATURES_ROUTE)}>
              <Flag className="size-4" />
              Open Features
            </Button>
          </CardContent>
        </Card>
        <Card size="sm">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Recent Activity</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <p className="text-2xl font-semibold">{events.length}</p>
            <Button size="sm" variant="outline" onClick={() => onNavigate(ACTIVITY_ROUTE)}>
              <ScrollText className="size-4" />
              View Activity
            </Button>
          </CardContent>
        </Card>
      </div>

      <Card size="sm">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm">Issue Workflow</CardTitle>
          <CardDescription>Status distribution for this project.</CardDescription>
        </CardHeader>
        <CardContent className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
          {statuses.map((status) => {
            const count = issues.filter((entry) => entry.status === status.id).length;
            const style = getStatusStyles(status);
            return (
              <div key={status.id} className="bg-muted/40 flex items-center justify-between rounded-md border px-3 py-2">
                <span className="inline-flex items-center gap-2 text-sm">
                  <span className={`text-xs ${style.color}`}>●</span>
                  {status.name}
                </span>
                <Badge variant="outline">{count}</Badge>
              </div>
            );
          })}
        </CardContent>
      </Card>

      <Card size="sm">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm">Quick Actions</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-wrap gap-2">
          <Button onClick={() => onNavigate(ISSUES_ROUTE)}>
            <FolderGit2 className="size-4" />
            Manage Issues
          </Button>
          <Button variant="outline" onClick={() => onNavigate(SPECS_ROUTE)}>
            <FileCode2 className="size-4" />
            Edit Specs
          </Button>
          <Button variant="outline" onClick={() => onNavigate(ADRS_ROUTE)}>
            <FileStack className="size-4" />
            Review ADRs
          </Button>
          <Button variant="outline" onClick={() => onNavigate(RELEASES_ROUTE)}>
            <Package className="size-4" />
            Track Releases
          </Button>
          <Button variant="outline" onClick={() => onNavigate(FEATURES_ROUTE)}>
            <Flag className="size-4" />
            Plan Features
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
