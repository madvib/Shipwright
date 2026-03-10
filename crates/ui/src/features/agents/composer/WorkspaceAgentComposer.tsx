import { useMemo } from 'react';
import { Check, ChevronDown, ChevronRight, Layers3, Shield, Sparkles } from 'lucide-react';
import { Badge, Button, Card, CardContent, Collapsible, CollapsibleContent, CollapsibleTrigger, Label } from '@ship/ui';
import { AutocompleteInput } from '@ship/ui';
import { cn } from '@/lib/utils';
import type { ComposerArtifact, ComposerSelection, ComposerTemplate } from './types';

type WorkspaceAgentComposerProps = {
  templates: ComposerTemplate[];
  artifacts: ComposerArtifact[];
  selection: ComposerSelection;
  onSelectionChange: (next: ComposerSelection) => void;
  className?: string;
};

function toOptions(values: ComposerArtifact[]) {
  return values.map((item) => ({
    value: item.id,
    label: item.name,
    keywords: [item.kind, item.scope, item.description ?? ''],
  }));
}

function nextList(current: string[], value: string): string[] {
  const trimmed = value.trim();
  if (!trimmed) return current;
  if (current.includes(trimmed)) return current;
  return [...current, trimmed];
}

function removeValue(current: string[], value: string): string[] {
  return current.filter((item) => item !== value);
}

export default function WorkspaceAgentComposer({
  templates,
  artifacts,
  selection,
  onSelectionChange,
  className,
}: WorkspaceAgentComposerProps) {
  const templateById = useMemo(
    () => new Map(templates.map((template) => [template.id, template])),
    [templates]
  );
  const selectedTemplate = selection.templateId
    ? (templateById.get(selection.templateId) ?? null)
    : null;

  const skills = artifacts.filter((artifact) => artifact.kind === 'skill');
  const mcpServers = artifacts.filter((artifact) => artifact.kind === 'mcp');
  const rules = artifacts.filter((artifact) => artifact.kind === 'rule');

  const skillOptions = toOptions(skills);
  const mcpOptions = toOptions(mcpServers);
  const ruleOptions = toOptions(rules);

  const inferredSkillHints = useMemo(() => {
    if (!selectedTemplate) return [];
    return selectedTemplate.recommendedSkills.filter((id) => skills.some((skill) => skill.id === id));
  }, [selectedTemplate, skills]);

  const inferredMcpHints = useMemo(() => {
    if (!selectedTemplate) return [];
    return selectedTemplate.recommendedMcpServers.filter((id) => mcpServers.some((server) => server.id === id));
  }, [selectedTemplate, mcpServers]);

  return (
    <div className={cn('grid gap-4 lg:grid-cols-[320px_minmax(0,1fr)]', className)}>
      <Card size="sm" className="overflow-hidden">
        <div className="flex items-center gap-2 border-b px-4 py-3">
          <Sparkles className="size-4 text-primary" />
          <div>
            <p className="text-sm font-semibold">Template Library</p>
            <p className="text-[11px] text-muted-foreground">Pick a Ship-first base, then compose artifacts.</p>
          </div>
        </div>
        <CardContent className="space-y-2 !pt-4">
          {templates.map((template) => {
            const selected = selection.templateId === template.id;
            return (
              <Collapsible key={template.id} defaultOpen={selected} className="rounded-md border">
                <CollapsibleTrigger asChild>
                  <button
                    type="button"
                    className={cn(
                      'flex w-full items-center justify-between gap-2 px-3 py-2 text-left transition-colors',
                      selected ? 'bg-primary/5' : 'hover:bg-muted/40'
                    )}
                    onClick={() => {
                      onSelectionChange({
                        ...selection,
                        templateId: template.id,
                      });
                    }}
                  >
                    <div className="min-w-0">
                      <p className="truncate text-xs font-semibold">{template.name}</p>
                      <p className="line-clamp-2 text-[10px] text-muted-foreground">{template.description}</p>
                    </div>
                    {selected ? <Check className="size-3.5 text-primary" /> : <ChevronRight className="size-3.5 text-muted-foreground" />}
                  </button>
                </CollapsibleTrigger>
                <CollapsibleContent>
                  <div className="space-y-2 border-t bg-muted/20 px-3 py-2">
                    <div className="flex flex-wrap gap-1">
                      {template.targetAgents.map((agent) => (
                        <Badge key={`${template.id}-${agent}`} variant="outline" className="text-[10px]">
                          {agent}
                        </Badge>
                      ))}
                    </div>
                    <p className="text-[10px] text-muted-foreground">
                      {template.recommendedSkills.length} skills • {template.recommendedMcpServers.length} MCP servers
                    </p>
                  </div>
                </CollapsibleContent>
              </Collapsible>
            );
          })}
        </CardContent>
      </Card>

      <Card size="sm" className="overflow-hidden">
        <div className="flex items-center gap-2 border-b px-4 py-3">
          <Layers3 className="size-4 text-cyan-500" />
          <div className="flex-1">
            <p className="text-sm font-semibold">Composer</p>
            <p className="text-[11px] text-muted-foreground">Assemble skills, MCP, rules, and tool policy for this workspace agent.</p>
          </div>
          {selectedTemplate && (
            <Badge variant="secondary" className="text-[10px]">
              {selectedTemplate.name}
            </Badge>
          )}
        </div>
        <CardContent className="grid gap-3 !pt-4">
          <div className="grid gap-3 md:grid-cols-[1fr_1fr]">
            <div className="space-y-1.5">
              <Label className="text-xs">Skills</Label>
              <AutocompleteInput
                value=""
                options={skillOptions}
                placeholder="Add skill by ID or name..."
                onValueChange={(value) => {
                  const next = nextList(selection.skillIds, value);
                  if (next !== selection.skillIds) {
                    onSelectionChange({ ...selection, skillIds: next });
                  }
                }}
              />
              <div className="flex flex-wrap gap-1">
                {selection.skillIds.map((id) => (
                  <Button
                    key={`skill-${id}`}
                    type="button"
                    variant="outline"
                    size="xs"
                    className="h-6 px-2 text-[10px]"
                    onClick={() => onSelectionChange({ ...selection, skillIds: removeValue(selection.skillIds, id) })}
                  >
                    {id}
                  </Button>
                ))}
              </div>
            </div>

            <div className="space-y-1.5">
              <Label className="text-xs">MCP Servers</Label>
              <AutocompleteInput
                value=""
                options={mcpOptions}
                placeholder="Add MCP server ID..."
                onValueChange={(value) => {
                  const next = nextList(selection.mcpServerIds, value);
                  if (next !== selection.mcpServerIds) {
                    onSelectionChange({ ...selection, mcpServerIds: next });
                  }
                }}
              />
              <div className="flex flex-wrap gap-1">
                {selection.mcpServerIds.map((id) => (
                  <Button
                    key={`mcp-${id}`}
                    type="button"
                    variant="outline"
                    size="xs"
                    className="h-6 px-2 text-[10px]"
                    onClick={() => onSelectionChange({ ...selection, mcpServerIds: removeValue(selection.mcpServerIds, id) })}
                  >
                    {id}
                  </Button>
                ))}
              </div>
            </div>
          </div>

          <div className="space-y-1.5">
            <Label className="text-xs">Rules</Label>
            <AutocompleteInput
              value=""
              options={ruleOptions}
              placeholder="Attach rule docs..."
              onValueChange={(value) => {
                const next = nextList(selection.ruleIds, value);
                if (next !== selection.ruleIds) {
                  onSelectionChange({ ...selection, ruleIds: next });
                }
              }}
            />
            <div className="flex flex-wrap gap-1">
              {selection.ruleIds.map((id) => (
                <Button
                  key={`rule-${id}`}
                  type="button"
                  variant="outline"
                  size="xs"
                  className="h-6 px-2 text-[10px]"
                  onClick={() => onSelectionChange({ ...selection, ruleIds: removeValue(selection.ruleIds, id) })}
                >
                  {id}
                </Button>
              ))}
            </div>
          </div>

          <div className="rounded-md border bg-muted/20 px-3 py-2">
            <div className="mb-1.5 flex items-center gap-1.5">
              <Shield className="size-3.5 text-emerald-500" />
              <p className="text-[11px] font-semibold">Inferred From Template</p>
              <ChevronDown className="size-3 text-muted-foreground" />
            </div>
            <p className="text-[10px] text-muted-foreground">
              Skills: {inferredSkillHints.join(', ') || 'None detected'}
            </p>
            <p className="text-[10px] text-muted-foreground">
              MCP: {inferredMcpHints.join(', ') || 'None detected'}
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
