import { Card, CardContent } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

export type AgentConfigScope = 'project' | 'global';

interface AgentScopeCardProps {
  scope: AgentConfigScope;
  hasProject: boolean;
  onScopeChange: (next: AgentConfigScope) => void;
}

export default function AgentScopeCard({ scope, hasProject, onScopeChange }: AgentScopeCardProps) {
  return (
    <Card size="sm">
      <CardContent className="grid gap-2 py-3 sm:grid-cols-[auto_minmax(0,240px)] sm:items-center sm:gap-3">
        <div className="space-y-0.5">
          <p className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">Scope</p>
          <p className="text-xs text-muted-foreground">Use global defaults or project override.</p>
        </div>
        <div className="space-y-1">
          <Label className="sr-only">Agent Config Scope</Label>
          <Select value={scope} onValueChange={(value) => onScopeChange((value as AgentConfigScope) ?? 'global')}>
            <SelectTrigger className="h-8 w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="global">Global (~/.ship)</SelectItem>
              <SelectItem value="project" disabled={!hasProject}>
                Project (.ship)
              </SelectItem>
            </SelectContent>
          </Select>
          {scope === 'project' && !hasProject && (
            <p className="text-xs text-destructive">Open a project to edit project-scoped agent config.</p>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
