import {
  ArrowRight,
  FileCode2,
  FileStack,
  Flag,
  FolderGit2,
  Folders,
  Package,
  ScrollText,
  Sparkles,
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
import { PageFrame, PageHeader } from '@/components/app/PageFrame';

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
  const accentStyles = {
    sky: {
      card: 'border-l-sky-400/55',
      action: 'border-l-sky-400/55 hover:bg-sky-500/10 hover:border-sky-400/75',
    },
    blue: {
      card: 'border-l-blue-400/55',
      action: 'border-l-blue-400/55 hover:bg-blue-500/10 hover:border-blue-400/75',
    },
    amber: {
      card: 'border-l-amber-400/55',
      action: 'border-l-amber-400/55 hover:bg-amber-500/10 hover:border-amber-400/75',
    },
    emerald: {
      card: 'border-l-emerald-400/55',
      action: 'border-l-emerald-400/55 hover:bg-emerald-500/10 hover:border-emerald-400/75',
    },
    cyan: {
      card: 'border-l-cyan-400/55',
      action: 'border-l-cyan-400/55 hover:bg-cyan-500/10 hover:border-cyan-400/75',
    },
    zinc: {
      card: 'border-l-zinc-400/55',
      action: 'border-l-zinc-400/55 hover:bg-zinc-500/10 hover:border-zinc-400/75',
    },
  } as const;

  type AccentKey = keyof typeof accentStyles;

  const doneStatusIds = statuses
    .filter((status) => /done|closed|complete/i.test(status.id))
    .map((status) => status.id);
  const openIssues = issues.filter((entry) => !doneStatusIds.includes(entry.status));
  const completedIssues = issues.filter((entry) => doneStatusIds.includes(entry.status));
  const completionPct = issues.length === 0 ? 0 : Math.round((completedIssues.length / issues.length) * 100);
  const summaryCards: {
    title: string;
    value: number | string;
    subtitle: string;
    route: AppRoutePath;
    cta: string;
    icon: typeof FolderGit2;
    accent: AccentKey;
  }[] = [
    {
      title: 'Open Issues',
      value: openIssues.length,
      subtitle: 'In non-done statuses',
      route: ISSUES_ROUTE,
      cta: 'Open Board',
      icon: FolderGit2,
      accent: 'sky',
    },
    {
      title: 'Specs',
      value: specs.length,
      subtitle: 'Planning documents',
      route: SPECS_ROUTE,
      cta: 'Open Specs',
      icon: FileCode2,
      accent: 'blue',
    },
    {
      title: 'ADRs',
      value: adrs.length,
      subtitle: 'Decision records',
      route: ADRS_ROUTE,
      cta: 'Open ADRs',
      icon: FileStack,
      accent: 'amber',
    },
    {
      title: 'Releases',
      value: releases.length,
      subtitle: 'Milestones tracked',
      route: RELEASES_ROUTE,
      cta: 'Open Releases',
      icon: Package,
      accent: 'emerald',
    },
    {
      title: 'Features',
      value: features.length,
      subtitle: 'Work slices planned',
      route: FEATURES_ROUTE,
      cta: 'Open Features',
      icon: Flag,
      accent: 'cyan',
    },
    {
      title: 'Recent Activity',
      value: events.length,
      subtitle: 'Recent events captured',
      route: ACTIVITY_ROUTE,
      cta: 'View Activity',
      icon: ScrollText,
      accent: 'zinc',
    },
    {
      title: 'Completion',
      value: `${completionPct}%`,
      subtitle: `${completedIssues.length} / ${issues.length} issues done`,
      route: ISSUES_ROUTE,
      cta: 'Review Issues',
      icon: FolderGit2,
      accent: 'emerald',
    },
  ];
  const quickActions: {
    label: string;
    route: AppRoutePath;
    icon: typeof FolderGit2;
    accent: AccentKey;
  }[] = [
    {
      label: 'Manage Issues',
      route: ISSUES_ROUTE,
      icon: FolderGit2,
      accent: 'sky',
    },
    {
      label: 'Edit Specs',
      route: SPECS_ROUTE,
      icon: FileCode2,
      accent: 'blue',
    },
    {
      label: 'Review ADRs',
      route: ADRS_ROUTE,
      icon: FileStack,
      accent: 'amber',
    },
    {
      label: 'Track Releases',
      route: RELEASES_ROUTE,
      icon: Package,
      accent: 'emerald',
    },
    {
      label: 'Plan Features',
      route: FEATURES_ROUTE,
      icon: Flag,
      accent: 'cyan',
    },
    {
      label: 'View Activity',
      route: ACTIVITY_ROUTE,
      icon: ScrollText,
      accent: 'zinc',
    },
  ];

  return (
    <PageFrame>
      <PageHeader
        title={<span title={project.path}>{project.name}</span>}
        description="Project overview and workflow status."
        actions={
          <Button variant="outline" title={project.path} onClick={() => onNavigate(PROJECTS_ROUTE)}>
            <Folders className="size-4" />
            Switch Project
          </Button>
        }
        footer={
          <div className="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-amber-400/35 bg-amber-500/[0.08] px-3 py-2">
            <div className="min-w-0">
              <p className="flex items-center gap-1.5 text-sm font-medium">
                <Sparkles className="size-3.5 text-amber-500" />
                Vision
              </p>
              <p className="text-muted-foreground text-xs">Project direction and intent.</p>
            </div>
            <Button size="xs" variant="outline" onClick={() => onNavigate(SPECS_ROUTE)}>
              <FileCode2 className="size-3.5" />
              Open
            </Button>
          </div>
        }
      />

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
          {quickActions.map((action) => {
            const Icon = action.icon;
            return (
              <Button
                key={action.label}
                size="xs"
                variant="outline"
                className={`bg-muted/30 border-l-4 transition-colors ${accentStyles[action.accent].action}`}
                onClick={() => onNavigate(action.route)}
              >
                <Icon className="size-3.5" />
                {action.label}
                <ArrowRight className="size-3.5" />
              </Button>
            );
          })}
        </CardContent>
      </Card>

      <div className="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
        {summaryCards.map((card) => {
          const Icon = card.icon;
          return (
            <Card key={card.title} size="sm" className={`border-l-4 ${accentStyles[card.accent].card}`}>
              <CardHeader className="py-1.5">
                <CardTitle className="text-sm">{card.title}</CardTitle>
              </CardHeader>
              <CardContent className="space-y-1 pt-0 pb-2.5">
                <p className="text-lg font-semibold">{card.value}</p>
                <p className="text-muted-foreground text-xs">{card.subtitle}</p>
                <Button size="xs" variant="outline" onClick={() => onNavigate(card.route)}>
                  <Icon className="size-3.5" />
                  {card.cta}
                  <ArrowRight className="size-3.5" />
                </Button>
              </CardContent>
            </Card>
          );
        })}
      </div>
    </PageFrame>
  );
}
