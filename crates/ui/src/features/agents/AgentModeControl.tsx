import { Bot, ChevronDown, Settings2 } from 'lucide-react';
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
import { cn } from '@/lib/utils';

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

const PROVIDER_LABELS: Record<string, string> = {
  claude: 'Claude',
  gemini: 'Gemini',
  codex: 'Codex',
};

function formatModelShort(model: string | null): string {
  if (!model) return '';
  // Trim common prefixes to keep label compact
  return model
    .replace(/^claude-/i, '')
    .replace(/^gemini-/i, '')
    .replace(/^gpt-/i, '')
    .split('-')
    .slice(0, 2)
    .join('-');
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
  const providerLabel = PROVIDER_LABELS[aiProvider ?? 'claude'] ?? (aiProvider ?? 'Claude');
  const modelShort = formatModelShort(aiModel);

  return (
    <div className="relative overflow-hidden rounded-xl border border-primary/25 bg-gradient-to-r from-primary/10 via-card/80 to-card/50 shadow-sm">
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_18%_40%,rgba(255,255,255,0.08),transparent_55%)]" />
      <div className="relative flex items-center gap-1 px-2 py-1">
        {/* Logo */}
        <div className="bg-primary/15 border-primary/30 flex size-6 shrink-0 items-center justify-center rounded-md border">
          <img src="/logo.svg" alt="Shipwright" className="size-4 object-contain" />
        </div>

        {/* Combined dropdown — mode + provider/model */}
        <DropdownMenu>
          <DropdownMenuTrigger
            render={
              <Button
                variant="ghost"
                size="xs"
                className="h-7 gap-1.5 px-2 text-xs font-medium hover:bg-primary/10"
              />
            }
          >
            <Bot className="size-3 text-muted-foreground" />
            <span className="text-foreground">{currentModeLabel}</span>
            {(providerLabel || modelShort) && (
              <span className={cn(
                'text-muted-foreground',
                switchingMode && 'animate-pulse'
              )}>
                · {providerLabel}{modelShort ? `/${modelShort}` : ''}
              </span>
            )}
            <ChevronDown className="size-3 text-muted-foreground" />
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-72">
            {/* Mode selection */}
            <DropdownMenuGroup>
              <DropdownMenuLabel className="text-[10px] font-black uppercase tracking-widest text-muted-foreground">
                Agent Mode
              </DropdownMenuLabel>
              <DropdownMenuRadioGroup
                value={activeModeId ?? DEFAULT_MODE_VALUE}
                onValueChange={(value) => onSetMode(value === DEFAULT_MODE_VALUE ? null : value)}
              >
                <DropdownMenuRadioItem value={DEFAULT_MODE_VALUE}>
                  <div className="flex flex-col">
                    <span>Default</span>
                    <span className="text-xs text-muted-foreground">All capabilities enabled</span>
                  </div>
                </DropdownMenuRadioItem>
                {modes.map((mode) => (
                  <DropdownMenuRadioItem key={mode.id} value={mode.id}>
                    <div className="flex flex-col">
                      <span>{mode.name}</span>
                      {mode.description && (
                        <span className="text-xs text-muted-foreground truncate max-w-[200px]">
                          {mode.description}
                        </span>
                      )}
                    </div>
                  </DropdownMenuRadioItem>
                ))}
              </DropdownMenuRadioGroup>
            </DropdownMenuGroup>

            <DropdownMenuSeparator />

            {/* Provider info */}
            <DropdownMenuGroup>
              <DropdownMenuLabel className="text-[10px] font-black uppercase tracking-widest text-muted-foreground">
                Current Agent
              </DropdownMenuLabel>
              <DropdownMenuItem className="gap-2" onClick={onOpenAgents}>
                <div className="flex min-w-0 flex-col">
                  <span className="text-xs font-medium">{providerLabel}</span>
                  {aiModel && (
                    <span className="text-xs text-muted-foreground font-mono truncate">{aiModel}</span>
                  )}
                  {!aiModel && (
                    <span className="text-xs text-muted-foreground italic">No model configured</span>
                  )}
                </div>
              </DropdownMenuItem>
            </DropdownMenuGroup>

            <DropdownMenuSeparator />
            <DropdownMenuGroup>
              <DropdownMenuItem onClick={onOpenAgents}>
                <Settings2 className="size-3.5" />
                {modes.length === 0 ? 'Create Agent Modes…' : 'Configure Agents…'}
              </DropdownMenuItem>
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  );
}
