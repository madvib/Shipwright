import { Label } from '@ship/primitives';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ship/primitives';

export type AgentConfigScope = 'project' | 'global';

interface AgentScopeCardProps {
  scope: AgentConfigScope;
  hasProject: boolean;
  onScopeChange: (next: AgentConfigScope) => void;
}

export default function AgentScopeCard({ scope, hasProject, onScopeChange }: AgentScopeCardProps) {
  return (
    <div className="rounded-md border px-2.5 py-2">
      <div className="flex flex-wrap items-center gap-2">
        <Label className="text-muted-foreground text-[11px] font-medium uppercase tracking-wide">Scope</Label>
        <Select value={scope} onValueChange={(value) => onScopeChange((value as AgentConfigScope) ?? 'global')}>
          <SelectTrigger size="sm" className="h-7 w-[190px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="global">Global</SelectItem>
            <SelectItem value="project" disabled={!hasProject}>
              Project
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
      {scope === 'project' && !hasProject && (
        <p className="mt-1 text-xs text-destructive">Open a project to use project scope.</p>
      )}
    </div>
  );
}
