import { BookOpen, Download, ExternalLink, Folder, PenLine, Plus, Save, ScrollText, Trash2 } from 'lucide-react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Badge } from '@ship/primitives';
import { Button } from '@ship/primitives';
import { Input } from '@ship/primitives';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@ship/primitives';
import { Tooltip, TooltipTrigger, TooltipContent } from '@ship/primitives';
import { cn } from '@/lib/utils';
import MarkdownEditor from '@/components/editor';
import { ExplorerDialog } from '@/features/agents/shared/ExplorerDialog';
import type { AgentDoc, AgentSection, MarkdownDocKind, ScopeKey } from '../agents.types';

export interface SkillsSectionProps {
  initialSection: 'skills' | 'rules';
  activeDocKind: MarkdownDocKind;
  agentScope: ScopeKey;
  activeDocs: AgentDoc[];
  activeDoc: AgentDoc | null;
  skillFolderRows: Array<{ id: string; fileName: string; title: string }>;
  skillExplorerOpen: boolean;
  setSkillExplorerOpen: (open: boolean) => void;
  skillSourceInput: string;
  setSkillSourceInput: (value: string) => void;
  parsedSkillInstallSpec: {
    source: string;
    skillId: string;
    parseHint: string | null;
    canInstall: boolean;
  };
  canInstallFromSource: boolean;
  installSkillFromSourceIsPending: boolean;
  installSkillFromSourceError: unknown;
  projectConfig: boolean;
  onSelectActiveDoc: (kind: MarkdownDocKind, docId: string) => void;
  onCreateDoc: (kind: MarkdownDocKind) => void;
  onDeleteDoc: (kind: MarkdownDocKind, docId: string) => void;
  onUpsertDoc: (kind: MarkdownDocKind, docId: string, patch: Partial<AgentDoc>) => void;
  onInstallSkillFromSource: () => void;
  onSave: () => void;
}

export function SkillsSection({
  initialSection,
  activeDocKind,
  agentScope,
  activeDocs,
  activeDoc,
  skillFolderRows,
  skillExplorerOpen,
  setSkillExplorerOpen,
  skillSourceInput,
  setSkillSourceInput,
  parsedSkillInstallSpec,
  canInstallFromSource,
  installSkillFromSourceIsPending,
  installSkillFromSourceError,
  projectConfig,
  onSelectActiveDoc,
  onCreateDoc,
  onDeleteDoc,
  onUpsertDoc,
  onInstallSkillFromSource,
  onSave,
}: SkillsSectionProps) {
  return (
    <div className="grid gap-4">
      <div className="grid gap-5 md:grid-cols-[320px_minmax(0,1fr)] xl:grid-cols-[360px_minmax(0,1fr)]">
        <div className="space-y-3">
          <div className="flex items-center gap-3 px-1">
            <div className="flex size-7 items-center justify-center rounded-lg border border-cyan-500/20 bg-cyan-500/10">
              {initialSection === 'skills' ? <BookOpen className="size-3.5 text-cyan-500" /> : <ScrollText className="size-3.5 text-cyan-500" />}
            </div>
            <div>
              <h3 className="text-sm font-semibold">{initialSection === 'skills' ? 'Skills' : 'Rules'}</h3>
              <p className="text-[11px] text-muted-foreground">{initialSection === 'skills' ? `${agentScope} scope` : 'global scope'}</p>
            </div>
          </div>

          <div className="grid gap-2 sm:grid-cols-2 md:grid-cols-1">
            <Button variant="outline" size="sm" className="w-full justify-start" onClick={() => onCreateDoc(activeDocKind)}>
              <Plus className="size-3.5" />
              New {initialSection === 'skills' ? 'Skill' : 'Rule'}
            </Button>
            {initialSection === 'skills' ? (
              <Button
                variant="outline"
                size="sm"
                className="w-full justify-start"
                onClick={() => setSkillExplorerOpen(true)}
              >
                <Download className="size-3.5" />
                Discover
              </Button>
            ) : null}
          </div>

          <div className="space-y-1">
            {activeDocs.length === 0 ? (
              <p className="py-4 text-center text-xs text-muted-foreground">
                No {initialSection === 'skills' ? 'skills' : 'rules'} yet.
              </p>
            ) : (
              initialSection === 'skills' ? (
                <div className="max-h-[62vh] overflow-y-auto rounded-lg border bg-background/60">
                  {skillFolderRows.map((skill) => {
                    const selected = activeDoc?.id === skill.id;
                    return (
                      <div key={skill.id} className="border-b last:border-b-0">
                        <button
                          type="button"
                          className={cn(
                            'flex w-full items-center gap-2 px-2.5 py-2 text-left text-xs font-medium',
                            selected ? 'bg-primary/10 text-primary' : 'hover:bg-muted/40'
                          )}
                          onClick={() => onSelectActiveDoc(activeDocKind, skill.id)}
                        >
                          <Folder className="size-3.5 opacity-80" />
                          <span className="truncate">{skill.id}</span>
                        </button>
                        <button
                          type="button"
                          className={cn(
                            'flex w-full items-center gap-2 px-2.5 py-1.5 pl-8 text-left text-xs',
                            selected ? 'bg-primary/5 text-primary' : 'text-muted-foreground hover:bg-muted/30'
                          )}
                          onClick={() => onSelectActiveDoc(activeDocKind, skill.id)}
                        >
                          <ScrollText className="size-3.5 opacity-70" />
                          <span>{skill.fileName}</span>
                          <span className="truncate opacity-70">· {skill.title}</span>
                        </button>
                      </div>
                    );
                  })}
                </div>
              ) : (
                activeDocs.map((doc) => {
                  const selected = activeDoc?.id === doc.id;
                  return (
                    <button
                      key={doc.id}
                      type="button"
                      className={cn(
                        'w-full rounded-md px-2.5 py-2 text-left transition-colors',
                        selected ? 'bg-primary/10 text-primary' : 'hover:bg-muted/50'
                      )}
                      onClick={() => onSelectActiveDoc(activeDocKind, doc.id)}
                    >
                      <p className="truncate text-sm font-medium">{doc.title || 'Untitled'}</p>
                    </button>
                  );
                })
              )
            )}
          </div>
        </div>

        <div className="space-y-3">
          <div className="flex items-center gap-3 px-1">
            <div className="flex size-7 items-center justify-center rounded-lg border border-indigo-500/20 bg-indigo-500/10">
              <PenLine className="size-3.5 text-indigo-500" />
            </div>
            <div className="flex-1">
              <div className="flex items-center gap-1.5 text-sm font-semibold">
                <span className="opacity-50 font-normal">{initialSection === 'skills' ? 'Skills' : 'Rules'}</span>
                <span className="opacity-30">/</span>
                <span>{initialSection === 'skills' ? 'Skill Editor' : 'Rules Editor'}</span>
                <Badge variant="outline" className="ml-2 h-4 px-1.5 py-0 text-[10px] font-normal normal-case tracking-tight opacity-70">
                  {agentScope} scope
                </Badge>
              </div>
              <p className="text-[11px] text-muted-foreground">{initialSection === 'skills' ? 'Edit skill content' : 'Edit rule content'}</p>
            </div>
            <Button
              variant="outline"
              size="xs"
              className="h-7"
              onClick={() => onSave()}
              disabled={agentScope === 'project' && !projectConfig}
            >
              <Save className="mr-1 size-3.5" />
              Save {agentScope === 'project' ? 'Project' : 'Global'}
            </Button>
            {activeDoc ? (
              <Button
                variant="ghost"
                size="xs"
                className="text-muted-foreground opacity-70 hover:bg-muted hover:text-destructive"
                onClick={() => onDeleteDoc(activeDocKind, activeDoc.id)}
              >
                <Trash2 className="mr-1 size-3.5" />
                Delete
              </Button>
            ) : null}
          </div>
          {!activeDoc ? (
            <div className="flex h-[440px] flex-col items-center justify-center gap-2 text-center">
              <ScrollText className="size-8 text-muted-foreground opacity-30" />
              <p className="text-sm text-muted-foreground">Select or create a document to start editing.</p>
            </div>
          ) : (
            <div className="space-y-3">
              <Input
                value={activeDoc.title}
                onChange={(event) => onUpsertDoc(activeDocKind, activeDoc.id, { title: event.target.value })}
                placeholder="Document title"
              />
              <MarkdownEditor
                label={undefined}
                value={activeDoc.content}
                onChange={(value) => onUpsertDoc(activeDocKind, activeDoc.id, { content: value })}
                placeholder={initialSection === 'skills' ? '# Skill' : '# Rule'}
                rows={36}
                className="min-h-[72vh]"
                editorClassName={initialSection === 'rules' ? '!border-0 !rounded-none !bg-transparent' : undefined}
                defaultMode="edit"
                showFrontmatter={false}
                showStats={false}
                fillHeight={false}
              />
            </div>
          )}
        </div>
      </div>

      {initialSection === 'skills' ? (
        <ExplorerDialog
          open={skillExplorerOpen}
          onOpenChange={setSkillExplorerOpen}
          title="Discover Skills"
          icon={<BookOpen className="size-4 text-cyan-500" />}
        >
          <Tabs defaultValue="skills-sh" className="flex h-full min-h-0 flex-col gap-3">
            <TabsList className="h-8 w-fit">
              <TabsTrigger value="skills-sh" className="text-xs">skills.sh</TabsTrigger>
              <TabsTrigger value="curated" className="text-xs">Curated Repo</TabsTrigger>
            </TabsList>

            <TabsContent value="skills-sh" className="min-h-0 flex-1">
              <div className="flex h-full min-h-0 flex-col gap-3 rounded-md border bg-muted/15 p-3">
                <div className="space-y-2">
                  <div className="grid gap-2 sm:grid-cols-[minmax(0,1fr)_auto_auto]">
                    <Input
                      value={skillSourceInput}
                      onChange={(event) => setSkillSourceInput(event.target.value)}
                      placeholder="Paste full skills.sh command or skill ID"
                      className="h-9 text-xs font-mono"
                      autoCapitalize="none"
                      autoCorrect="off"
                      spellCheck={false}
                    />
                    <Button
                      type="button"
                      size="sm"
                      variant="secondary"
                      className="h-9"
                      onClick={() => onInstallSkillFromSource()}
                      disabled={!canInstallFromSource || installSkillFromSourceIsPending}
                    >
                      {installSkillFromSourceIsPending ? 'Installing…' : (
                        <>
                          <Download className="mr-1 size-3.5" />
                          Install
                        </>
                      )}
                    </Button>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant="outline"
                          size="icon-sm"
                          aria-label="Open skills.sh externally"
                          onClick={() => {
                            void openUrl('https://skills.sh');
                          }}
                        >
                          <ExternalLink className="size-3.5" />
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>Open skills.sh (external)</TooltipContent>
                    </Tooltip>
                  </div>
                  <p className="text-[11px] text-muted-foreground">
                    {parsedSkillInstallSpec.parseHint ?? 'Use `npx skills add <skill-id>` or just `<skill-id>`.'}
                  </p>
                  {installSkillFromSourceError ? (
                    <p className="text-[11px] text-destructive">{String(installSkillFromSourceError)}</p>
                  ) : null}
                </div>
              </div>
            </TabsContent>

            <TabsContent value="curated" className="min-h-0 flex-1">
              <div className="flex h-full min-h-0 flex-col items-center justify-center gap-2 rounded-md border border-dashed bg-muted/15 p-4 text-center">
                <BookOpen className="size-5 text-muted-foreground/70" />
                <p className="text-sm font-medium">Curated repository not connected yet.</p>
                <p className="text-xs text-muted-foreground">
                  This tab will host Ship-curated skills as a first-class discovery source.
                </p>
              </div>
            </TabsContent>
          </Tabs>
        </ExplorerDialog>
      ) : null}
    </div>
  );
}
