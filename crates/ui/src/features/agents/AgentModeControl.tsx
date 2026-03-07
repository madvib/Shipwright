import { Bot, Check, ChevronDown, Settings2, Sparkles } from 'lucide-react';
import { ModeConfig } from '@/bindings';
import { Button, Badge } from '@ship/ui';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@ship/ui';

interface AgentModeControlProps {
  modes: ModeConfig[];
  activeModeId: string | null;
  aiProvider: string | null;
  aiModel: string | null;
  switchingMode: boolean;
  onSetMode: (modeId: string | null) => void;
  onOpenAgents: () => void;
}

const DEFAULT_MODE_VALUE = '__default__';

export default function AgentModeControl({
  modes,
  activeModeId,
  aiProvider,
  aiModel,
  switchingMode,
  onSetMode,
  onOpenAgents,
}: AgentModeControlProps) {
  const activeMode = modes.find((mode) => mode.id === activeModeId) ?? null;
  const activeLabel = activeMode?.name ?? 'Default';

  return (
    <div className="flex items-center gap-2">
      <DropdownMenu>
        <DropdownMenuTrigger
          render={
            <Button
              variant="outline"
              size="sm"
              className="h-8 gap-2 px-3 text-xs font-medium"
            />
          }
        >
          <Sparkles className="size-3.5 text-primary" />
          <span className="max-w-[12rem] truncate">{activeLabel}</span>
          <ChevronDown className="size-3.5 text-muted-foreground" />
        </DropdownMenuTrigger>
        <DropdownMenuContent
          align="end"
          className="w-80 border-border bg-popover p-1"
        >
          <div className="px-2 py-1.5">
            <p className="text-[11px] font-semibold text-foreground">Agent Mode</p>
            <p className="text-[10px] text-muted-foreground">
              Pick a default mode. Create and edit modes in agent settings.
            </p>
          </div>

          <DropdownMenuRadioGroup
            value={activeModeId ?? DEFAULT_MODE_VALUE}
            onValueChange={(value) =>
              onSetMode(value === DEFAULT_MODE_VALUE ? null : value)
            }
          >
            <DropdownMenuRadioItem
              value={DEFAULT_MODE_VALUE}
              className="rounded-md px-2 py-2"
            >
              <div className="flex min-w-0 items-center justify-between gap-2">
                <div className="flex min-w-0 items-center gap-2">
                  <Bot className="size-3.5 text-muted-foreground" />
                  <div className="min-w-0">
                    <p className="truncate text-xs font-medium">Default</p>
                    <p className="truncate text-[10px] text-muted-foreground">
                      Use project default behavior
                    </p>
                  </div>
                </div>
                {activeModeId === null ? <Check className="size-3.5 text-primary" /> : null}
              </div>
            </DropdownMenuRadioItem>

            {modes.map((mode) => (
              <DropdownMenuRadioItem
                key={mode.id}
                value={mode.id}
                className="rounded-md px-2 py-2"
              >
                <div className="flex min-w-0 items-center justify-between gap-2">
                  <div className="min-w-0">
                    <p className="truncate text-xs font-medium">{mode.name}</p>
                    {mode.description ? (
                      <p className="truncate text-[10px] text-muted-foreground">
                        {mode.description}
                      </p>
                    ) : null}
                  </div>
                  {activeModeId === mode.id ? (
                    <Check className="size-3.5 text-primary" />
                  ) : null}
                </div>
              </DropdownMenuRadioItem>
            ))}
          </DropdownMenuRadioGroup>

          <DropdownMenuSeparator />
          <div className="flex items-center justify-between px-2 py-2">
            <div className="min-w-0">
              <p className="text-[10px] text-muted-foreground">
                Provider: <span className="font-medium text-foreground">{aiProvider ?? 'none'}</span>
              </p>
              <p className="truncate text-[10px] text-muted-foreground">
                Model: {aiModel ?? 'not set'}
              </p>
            </div>
            <Button
              variant="ghost"
              size="xs"
              className="h-6 gap-1 px-2 text-[10px]"
              onClick={(event) => {
                event.preventDefault();
                event.stopPropagation();
                onOpenAgents();
              }}
            >
              <Settings2 className="size-3" />
              Manage
            </Button>
          </div>
        </DropdownMenuContent>
      </DropdownMenu>

      {switchingMode ? (
        <Badge variant="secondary" className="h-6 text-[10px]">
          updating...
        </Badge>
      ) : null}
    </div>
  );
}
