import { useMemo, useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { commands } from '@/bindings';
import MarkdownEditor from '@/components/editor';

import {
  FileStack,
  Folders,
  Package,
  ScrollText,
  Sparkles,
  Target,
  Lightbulb,
  Plus,
  History,
} from 'lucide-react';
import { splitFrontmatterDocument } from '@ship/ui';
import { AdrHeaderMetadata } from '../adrs/AdrHeaderMetadata';

import {
  AdrEntry,
  EventRecord,
  FeatureInfo as FeatureEntry,
  NoteInfo as NoteEntry,
  ProjectDiscovery as Project,
  ReleaseInfo as ReleaseEntry,
} from '@/bindings';
import { SpecInfo as SpecEntry } from '@/lib/types/spec';
import {
  ADRS_ROUTE,
  AppRoutePath,
  FEATURES_ROUTE,
  NOTES_ROUTE,
  PROJECTS_ROUTE,
  RELEASES_ROUTE,
  ACTIVITY_ROUTE,
} from '@/lib/constants/routes';
import { Badge } from '@ship/ui';
import { Button } from '@ship/ui';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@ship/ui';
import { PageFrame, PageHeader, DetailSheet } from '@ship/ui';

interface ProjectOverviewProps {
  project: Project;
  specs: SpecEntry[];
  adrs: AdrEntry[];
  releases: ReleaseEntry[];
  features: FeatureEntry[];
  notes: NoteEntry[];
  events: EventRecord[];
  onNavigate: (path: AppRoutePath) => void;
}

export default function ProjectOverview({
  project,
  specs,
  adrs,
  releases,
  features,
  notes,
  events,
  onNavigate,
}: ProjectOverviewProps) {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [visionEditing, setVisionEditing] = useState(false);
  const [visionDraft, setVisionDraft] = useState('');

  const { data: visionData } = useQuery({
    queryKey: ['vision'],
    queryFn: async () => {
      const res = await commands.getVisionCmd();
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
  });

  const updateVision = useMutation({
    mutationFn: async (content: string) => {
      const res = await commands.updateVisionCmd(content);
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vision'] });
      setVisionEditing(false);
    },
  });

  const handleVisionEdit = () => {
    setVisionDraft(visionData?.content ?? '');
    setVisionEditing(true);
  };

  const handleVisionSave = () => {
    updateVision.mutate(visionDraft);
  };

  const handleVisionCancel = () => {
    setVisionEditing(false);
  };

  const visionBody = useMemo(() => {
    if (!visionData?.content) return '';
    const { body } = splitFrontmatterDocument(visionData.content);

    // Strip markdown symbols for the preview
    return body
      .replace(/^#+\s+/gm, '') // Remove headers
      .replace(/[*_~`]/g, '') // Remove simple formatting
      .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1') // Remove links
      .trim();
  }, [visionData?.content]);

  const activeRelease = releases.find(r => r.status === 'active') || releases[0];
  const activeFeatures = activeRelease ? features.filter(f =>
    f.release_id === activeRelease.file_name ||
    (activeRelease.version && f.release_id === activeRelease.version)
  ) : [];

  const recentNotes = [...notes]
    .sort((a, b) => new Date(b.updated).getTime() - new Date(a.updated).getTime())
    .slice(0, 3);

  const recentAdrs = [...adrs]
    .slice(0, 3);
  const specCount = specs.length;

  return (
    <PageFrame>
      <PageHeader
        title={<span title={project.path}>{project.name}</span>}
        actions={
          <Button variant="outline" title={project.path} onClick={() => onNavigate(PROJECTS_ROUTE)}>
            <Folders className="size-4" />
            Switch Project
          </Button>
        }
      />

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        {/* Planning Loop Column */}
        <div className="space-y-6">
          <Card className="border-l-4 border-l-primary/60">
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <CardTitle className="text-base flex items-center gap-2">
                  <Target className="size-4 text-primary" />
                  Planning Loop
                </CardTitle>
                <Badge variant="outline" className="text-[10px] uppercase font-bold tracking-tighter opacity-70">Vision → Release → Feature</Badge>
              </div>
              <CardDescription>Multi-milestone project roadmap.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Vision Card (Compact) */}
              <div className="rounded-md border border-amber-500/20 bg-amber-500/5 p-3">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2 text-xs font-semibold text-amber-600">
                    <Sparkles className="size-3" />
                    PROJECT VISION
                  </div>
                  <Button size="xs" variant="ghost" className="h-6 px-2 text-[10px]" onClick={handleVisionEdit}>View Vision</Button>
                </div>
                <p className="text-xs text-muted-foreground line-clamp-2 leading-relaxed italic">
                  {visionBody || "No vision defined yet. What is your goal?"}
                </p>
              </div>

              {/* Release Tracking */}
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <h4 className="text-xs font-bold text-muted-foreground uppercase tracking-wider">Next Release</h4>
                  <Button variant="link" size="xs" className="h-auto p-0 text-[10px]" onClick={() => onNavigate(RELEASES_ROUTE)}>View All</Button>
                </div>
                {activeRelease ? (
                  <div className="rounded-md border p-3 flex items-center justify-between bg-card/50">
                    <div className="flex items-center gap-3">
                      <Package className="size-4 text-primary" />
                      <div>
                        <p className="text-sm font-bold">{activeRelease.version}</p>
                        <p className="text-[10px] text-muted-foreground">{activeRelease.status}</p>
                      </div>
                    </div>
                    <Badge variant="outline" className="text-[10px]">{activeFeatures.length} Features</Badge>
                  </div>
                ) : (
                  <div className="rounded-md border border-dashed p-6 text-center bg-muted/10">
                    <p className="text-xs text-muted-foreground mb-3">No releases planned.</p>
                    <Button size="xs" onClick={() => onNavigate(RELEASES_ROUTE)}>Start a Release</Button>
                  </div>
                )}
              </div>

              {/* Feature Preview */}
              <div className="space-y-3 pt-2">
                <div className="flex items-center justify-between">
                  <h4 className="text-xs font-bold text-muted-foreground uppercase tracking-wider">Active Features</h4>
                  <Button variant="link" size="xs" className="h-auto p-0 text-[10px]" onClick={() => onNavigate(FEATURES_ROUTE)}>All Features</Button>
                </div>
                <div className="grid gap-2">
                  {activeFeatures.slice(0, 3).map(feature => (
                    <div key={feature.file_name} className="flex items-center justify-between rounded-md border bg-card/30 px-3 py-2 text-xs">
                      <span className="font-medium truncate max-w-[180px]">{feature.title}</span>
                      <Badge variant="secondary" className="text-[9px] uppercase tracking-tight">{feature.status}</Badge>
                    </div>
                  ))}
                  {activeFeatures.length > 3 && (
                    <p className="text-[10px] text-center text-muted-foreground italic">+ {activeFeatures.length - 3} more features</p>
                  )}
                  {activeFeatures.length === 0 && activeRelease && (
                    <div className="rounded-md border border-dashed py-8 text-center bg-muted/5">
                      <Lightbulb className="size-5 text-muted-foreground/40 mx-auto mb-2" />
                      <p className="text-[10px] text-muted-foreground mb-3 font-medium">No features mapped to this release.</p>
                      <Button size="xs" variant="outline" className="h-7 text-[10px]" onClick={() => onNavigate(FEATURES_ROUTE)}>Add First Feature</Button>
                    </div>
                  )}
                </div>
              </div>
            </CardContent>
          </Card>

          {/* ADRs / Context */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-base flex items-center gap-2">
                <FileStack className="size-4 text-amber-500" />
                Context (ADRs)
              </CardTitle>
              <CardDescription>Architectural decision records.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="text-muted-foreground text-[10px] uppercase tracking-wider">
                {specCount} spec{specCount === 1 ? '' : 's'} in context
              </div>
              {recentAdrs.map(adr => (
                <div key={adr.file_name} className="flex flex-col gap-1.5 rounded-md border bg-card/30 px-3 py-2 text-xs">
                  <div className="flex items-center justify-between">
                    <span className="font-medium truncate">{adr.adr.metadata.title}</span>
                    <Badge variant="outline" className="text-[9px] uppercase tracking-tight">{adr.status}</Badge>
                  </div>
                  <AdrHeaderMetadata
                    adr={adr.adr}
                    onChange={() => { }}
                    specSuggestions={specs.map(s => ({ id: s.id, title: s.spec.metadata.title }))}
                    adrSuggestions={adrs.map(a => ({ id: a.file_name, title: a.adr.metadata.title }))}
                    tagSuggestions={[]}
                    onNavigate={(type, id) => {
                      if (type === 'adr') {
                        navigate({ to: ADRS_ROUTE, search: { id } });
                      }
                    }}
                  />
                </div>
              ))}
              {recentAdrs.length === 0 && (
                <p className="text-xs text-muted-foreground text-center py-4 italic">No decisions recorded.</p>
              )}
              <Button variant="outline" size="xs" className="w-full mt-2" onClick={() => onNavigate(ADRS_ROUTE)}>
                Open Decision Register
              </Button>
            </CardContent>
          </Card>
        </div>

        {/* Task Loop Column */}
        <div className="space-y-6">
          {/* Inbox / Brain Dump */}
          <Card className="border-l-4 border-l-emerald-500/60">
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <CardTitle className="text-base flex items-center gap-2">
                  <ScrollText className="size-4 text-emerald-500" />
                  Inbox (Notes)
                </CardTitle>
                <Button size="xs" variant="ghost" onClick={() => onNavigate(NOTES_ROUTE)}>
                  <Plus className="size-3.5" />
                </Button>
              </div>
              <CardDescription>Brain dump and quick thoughts.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-2">
              {recentNotes.map(note => (
                <div key={note.id} className="flex flex-col gap-1 rounded-md border bg-card/30 px-3 py-2 transition-colors hover:bg-accent/30 cursor-pointer" onClick={() => onNavigate(NOTES_ROUTE)}>
                  <span className="text-xs font-bold truncate">{note.title || 'Untitled'}</span>
                  <span className="text-[10px] text-muted-foreground">{new Date(note.updated).toLocaleDateString()}</span>
                </div>
              ))}
              {recentNotes.length === 0 && (
                <div className="rounded-md border border-dashed py-8 text-center bg-muted/5">
                  <Lightbulb className="size-5 text-muted-foreground mx-auto mb-2 opacity-50" />
                  <p className="text-xs text-muted-foreground">Your inbox is empty.</p>
                  <Button variant="link" size="xs" onClick={() => onNavigate(NOTES_ROUTE)}>Capture a thought</Button>
                </div>
              )}
            </CardContent>
          </Card>

          {/* Activity */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-base flex items-center gap-2">
                <History className="size-4" />
                Recent Activity
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="space-y-3">
                {events.slice(0, 3).map((event, idx) => (
                  <div key={idx} className="flex gap-3 text-[11px] leading-snug">
                    <div className="mt-1 size-1.5 shrink-0 rounded-full bg-primary/40" />
                    <div className="flex flex-col">
                      <span className="text-muted-foreground shrink-0 font-bold uppercase text-[9px] mr-2">{event.action} {event.entity}</span>
                      <span className="text-muted-foreground truncate">{event.subject}</span>
                      <span className="text-[9px] opacity-60 font-mono">{new Date(event.timestamp).toLocaleTimeString()}</span>
                    </div>
                  </div>
                ))}
              </div>
              <Button variant="ghost" size="xs" className="w-full mt-3 text-[10px]" onClick={() => onNavigate(ACTIVITY_ROUTE)}>Full Project Log</Button>
            </CardContent>
          </Card>
        </div>
      </div>

      {visionEditing && (
        <DetailSheet
          title={
            <div className="flex items-center gap-2">
              <Sparkles className="size-5 text-amber-500" />
              <h2 className="text-xl font-semibold tracking-tight">Project Vision</h2>
            </div>
          }
          onClose={handleVisionCancel}
          className="max-w-[1800px]"
          bodyScrollable={false}
          bodyClassName="overflow-hidden p-0"
          footer={
            <div className="flex justify-end gap-3 px-1">
              <Button variant="outline" onClick={handleVisionCancel}>
                Cancel
              </Button>
              <Button onClick={handleVisionSave} disabled={updateVision.isPending}>
                {updateVision.isPending ? 'Saving…' : 'Save Changes'}
              </Button>
            </div>
          }
        >
          <div className="h-full min-h-0 p-2">
            <MarkdownEditor
              value={visionDraft}
              onChange={setVisionDraft}
              fillHeight={true}
              showFrontmatter={false}
              placeholder="Write your project's north star — the single paragraph that explains why this project matters and where it's going."
              label={undefined}
            />
          </div>
        </DetailSheet>
      )}
    </PageFrame>
  );
}
