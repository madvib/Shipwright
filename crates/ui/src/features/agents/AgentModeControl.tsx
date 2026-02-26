import { Settings2 } from 'lucide-react';
import { ModeConfig } from '@/bindings';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

interface AgentModeControlProps {
  modes: ModeConfig[];
  activeModeId: string | null;
  aiProvider: string | null;
  aiModel: string | null;
  switchingMode: boolean;
  onSetMode: (modeId: string | null) => void;
  onOpenAgents: () => void;
}

const DEFAULT_MODE_VALUE = 'default';

function formatProviderName(provider: string | null): string {
  switch ((provider ?? 'claude').toLowerCase()) {
    case 'claude':
      return 'Claude';
    case 'gemini':
      return 'Gemini';
    case 'codex':
      return 'Codex';
    default:
      return provider ?? 'Claude';
  }
}

export default function AgentModeControl({
  modes,
  activeModeId,
  aiProvider,
  aiModel,
  switchingMode,
  onSetMode,
  onOpenAgents,
}: AgentModeControlProps) {
  const currentMode = modes.find((mode) => mode.id === activeModeId) ?? null;
  const currentModeLabel = currentMode?.name ?? 'Default';
  const providerLabel = formatProviderName(aiProvider);
  const modelLabel = aiModel?.trim() || 'Model not set';

  return (
    <div className="relative overflow-hidden rounded-xl border border-primary/25 bg-gradient-to-r from-primary/10 via-card/80 to-card/50 px-2 py-1.5 shadow-sm">
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_18%_40%,rgba(255,255,255,0.1),transparent_55%)]" />
      <div className="relative flex items-center gap-2">
        <div className="bg-primary/15 border-primary/30 flex size-7 items-center justify-center rounded-md border">
          <img src="/logo.svg" alt="Shipwright" className="size-5 object-contain" />
        </div>

        <DropdownMenu>
          <DropdownMenuTrigger
            render={
              <Button variant="outline" size="xs" className="h-8 bg-background/70 text-xs font-medium" />
            }
          >
            Agent Mode: {currentModeLabel}
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-72">
            <DropdownMenuGroup>
              <DropdownMenuLabel>Select Agent Mode</DropdownMenuLabel>
              <DropdownMenuRadioGroup
                value={activeModeId ?? DEFAULT_MODE_VALUE}
                onValueChange={(value) => onSetMode(value === DEFAULT_MODE_VALUE ? null : value)}
              >
                <DropdownMenuRadioItem value={DEFAULT_MODE_VALUE}>
                  Default (all capabilities)
                </DropdownMenuRadioItem>
                {modes.map((mode) => (
                  <DropdownMenuRadioItem key={mode.id} value={mode.id}>
                    {mode.name}
                  </DropdownMenuRadioItem>
                ))}
              </DropdownMenuRadioGroup>
            </DropdownMenuGroup>
            <DropdownMenuSeparator />
            <DropdownMenuGroup>
              <DropdownMenuItem onClick={onOpenAgents}>
                {modes.length === 0 ? 'Create Modes in Agents Module' : 'Configure Modes'}
              </DropdownMenuItem>
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>

        <DropdownMenu>
          <DropdownMenuTrigger
            render={
              <Button variant="outline" size="xs" className="h-8 bg-background/70 text-xs font-medium" />
            }
          >
            Current Agent: {providerLabel}
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-72">
            <DropdownMenuGroup>
              <DropdownMenuLabel>Agent Selection</DropdownMenuLabel>
              <DropdownMenuItem className="flex-col items-start gap-0.5">
                <span className="text-xs font-medium">Provider: {providerLabel}</span>
                <span className="text-muted-foreground text-xs">Model: {modelLabel}</span>
              </DropdownMenuItem>
              {switchingMode && (
                <DropdownMenuItem className="text-xs">Mode switch in progress...</DropdownMenuItem>
              )}
            </DropdownMenuGroup>
            <DropdownMenuSeparator />
            <DropdownMenuGroup>
              <DropdownMenuItem onClick={onOpenAgents}>
                <Settings2 className="size-3.5" />
                Configure Agent + Model
              </DropdownMenuItem>
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  );
}
