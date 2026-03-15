import { Info, Plus, Terminal, Trash2 } from 'lucide-react';
import { Badge } from '@ship/primitives';
import { Button } from '@ship/primitives';
import { Card, CardContent } from '@ship/primitives';
import { Input } from '@ship/primitives';
import { Label } from '@ship/primitives';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ship/primitives';
import { Tooltip, TooltipTrigger, TooltipContent } from '@ship/primitives';
import { AutocompleteInput } from '@ship/primitives';
import type { HookConfig } from '@/bindings';
import type { HookEventOption, ScopeKey } from '../agents.types';
import { HOOK_EVENTS } from '../agents.types';

export interface HooksSectionProps {
  hooks: HookConfig[];
  agentScope: ScopeKey;
  activeHookEvents: HookEventOption[];
  defaultHookTrigger: string;
  hookCommandSuggestions: Array<{ value: string }>;
  hookMatcherSuggestions: Array<{ value: string }>;
  providersWithNativeHooks: string[];
  providersWithoutNativeHooks: string[];
  onAddHook: () => void;
  onUpdateHook: (idx: number, patch: Partial<HookConfig>) => void;
  onRemoveHook: (idx: number) => void;
}

export function HooksSection({
  hooks,
  agentScope,
  activeHookEvents,
  defaultHookTrigger,
  hookCommandSuggestions,
  hookMatcherSuggestions,
  providersWithNativeHooks,
  providersWithoutNativeHooks,
  onAddHook,
  onUpdateHook,
  onRemoveHook,
}: HooksSectionProps) {
  return (
    <div className="grid gap-4 lg:grid-cols-[1fr_320px]">
      <Card size="sm" className="overflow-hidden">
        <div className="flex items-center gap-3 border-b bg-gradient-to-r from-amber-500/10 via-card/80 to-card/50 px-4 py-3">
          <div className="flex size-7 items-center justify-center rounded-lg border border-amber-500/20 bg-amber-500/10">
            <Terminal className="size-3.5 text-amber-500" />
          </div>
          <div className="flex-1">
            <div className="flex items-center gap-2">
              <h3 className="text-sm font-semibold">Lifecycle Hooks</h3>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Info className="size-3 cursor-default text-muted-foreground" />
                </TooltipTrigger>
                <TooltipContent>
                  Hooks export natively to Claude and Gemini. Codex stores hook config in Ship but has no native hook runtime yet.
                </TooltipContent>
              </Tooltip>
            </div>
            <p className="text-[11px] text-muted-foreground">
              Run command interceptors at key agent lifecycle moments for context, guardrails, and telemetry.
            </p>
          </div>
          <Badge variant="secondary" className="shrink-0 text-[10px]">
            {hooks.length} hook{hooks.length !== 1 ? 's' : ''}
          </Badge>
        </div>

        <CardContent className="space-y-3 !pt-5">
          {hooks.length === 0 && (
            <div className="rounded-lg border border-dashed p-6 text-center">
              <p className="text-sm text-muted-foreground">No hooks configured yet.</p>
              <p className="mt-1 text-[11px] text-muted-foreground/70">
                Add one to inject context, enforce shell policy, or stream events to ops.
              </p>
            </div>
          )}

          {hooks.map((hook, idx) => {
            const triggerValue = String(hook.trigger || defaultHookTrigger);
            const triggerMeta = HOOK_EVENTS.find((event) => event.value === triggerValue);
            return (
              <div key={`${hook.id}-${idx}`} className="space-y-3 rounded-lg border p-3">
                <div className="grid gap-2 sm:grid-cols-[1fr_180px_auto]">
                  <div className="space-y-1">
                    <div className="flex items-center gap-1.5">
                      <Label className="text-[11px]">Hook ID</Label>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Info className="size-3 cursor-default text-muted-foreground" />
                        </TooltipTrigger>
                        <TooltipContent>Stable ID for this hook in project config and exports.</TooltipContent>
                      </Tooltip>
                    </div>
                    <Input
                      value={hook.id ?? ''}
                      onChange={(e) => onUpdateHook(idx, { id: e.target.value })}
                      placeholder="hook-id"
                      className="h-8 text-xs font-mono"
                    />
                  </div>
                  <div className="space-y-1">
                    <div className="flex items-center gap-1.5">
                      <Label className="text-[11px]">Event</Label>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Info className="size-3 cursor-default text-muted-foreground" />
                        </TooltipTrigger>
                        <TooltipContent>Lifecycle moment that triggers this command.</TooltipContent>
                      </Tooltip>
                    </div>
                    <Select
                      value={triggerValue}
                      onValueChange={(value) =>
                        onUpdateHook(idx, { trigger: value as HookConfig['trigger'] })
                      }
                    >
                      <SelectTrigger size="sm">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {activeHookEvents.map((event) => (
                          <SelectItem key={event.value} value={event.value}>
                            {event.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-1">
                    <div className="flex items-center justify-end">
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            type="button"
                            variant="ghost"
                            size="xs"
                            className="h-8 w-8 p-0 text-destructive hover:bg-destructive/10 hover:text-destructive"
                            onClick={() => onRemoveHook(idx)}
                          >
                            <Trash2 className="size-3.5" />
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>Delete hook</TooltipContent>
                      </Tooltip>
                    </div>
                  </div>
                </div>

                <div className="space-y-1">
                  <div className="flex items-center gap-1.5">
                    <Label className="text-[11px]">Command</Label>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Info className="size-3 cursor-default text-muted-foreground" />
                      </TooltipTrigger>
                      <TooltipContent>Command executed when this hook fires.</TooltipContent>
                    </Tooltip>
                  </div>
                  <AutocompleteInput
                    value={hook.command ?? ''}
                    options={hookCommandSuggestions}
                    onValueChange={(value) => onUpdateHook(idx, { command: value })}
                    placeholder="$SHIP_HOOKS_BIN"
                    className="h-8 text-xs font-mono"
                  />
                </div>

                <div className="grid gap-2 sm:grid-cols-[1fr_140px_1fr]">
                  <div className="space-y-1">
                    <div className="flex items-center gap-1.5">
                      <Label className="text-[11px]">Description</Label>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Info className="size-3 cursor-default text-muted-foreground" />
                        </TooltipTrigger>
                        <TooltipContent>Optional note for audit logs and UI context.</TooltipContent>
                      </Tooltip>
                    </div>
                    <Input
                      value={hook.description ?? ''}
                      onChange={(e) => onUpdateHook(idx, { description: e.target.value || null })}
                      placeholder="Description (optional)"
                      className="h-8 text-xs"
                    />
                  </div>
                  <div className="space-y-1">
                    <div className="flex items-center gap-1.5">
                      <Label className="text-[11px]">Timeout</Label>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Info className="size-3 cursor-default text-muted-foreground" />
                        </TooltipTrigger>
                        <TooltipContent>Max runtime in milliseconds before the hook is aborted.</TooltipContent>
                      </Tooltip>
                    </div>
                    <Input
                      type="number"
                      min={0}
                      value={hook.timeout_ms ?? ''}
                      onChange={(e) => {
                        const raw = e.target.value.trim();
                        const parsed = Number(raw);
                        onUpdateHook(idx, {
                          timeout_ms: raw && Number.isFinite(parsed) ? parsed : null,
                        });
                      }}
                      placeholder="Timeout ms"
                      className="h-8 text-xs font-mono"
                    />
                  </div>
                  <div className="space-y-1">
                    <div className="flex items-center gap-1.5">
                      <Label className="text-[11px]">Matcher</Label>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Info className="size-3 cursor-default text-muted-foreground" />
                        </TooltipTrigger>
                        <TooltipContent>Optional tool/event filter. Leave blank to run on all matches.</TooltipContent>
                      </Tooltip>
                    </div>
                    <AutocompleteInput
                      value={hook.matcher ?? ''}
                      options={hookMatcherSuggestions}
                      onValueChange={(value) => onUpdateHook(idx, { matcher: value || null })}
                      placeholder={triggerMeta?.matcherHint ?? 'Matcher (optional)'}
                      className="h-8 text-xs font-mono"
                    />
                  </div>
                </div>
              </div>
            );
          })}

          <Tooltip>
            <TooltipTrigger asChild>
              <Button variant="outline" size="sm" className="w-full border-dashed" onClick={onAddHook}>
                <Plus className="mr-1.5 size-3.5" />
                Add Hook
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              Add a lifecycle hook command for context injection, policy, or logging.
            </TooltipContent>
          </Tooltip>
        </CardContent>
      </Card>

      <Card size="sm" className="h-fit overflow-hidden bg-muted/10">
        <div className="flex items-center gap-3 border-b bg-gradient-to-r from-slate-500/10 via-card/80 to-card/50 px-4 py-3">
          <div className="flex size-7 items-center justify-center rounded-lg border border-slate-500/20 bg-slate-500/10">
            <Info className="size-3.5 text-slate-500" />
          </div>
          <h3 className="text-sm font-semibold">Provider Support</h3>
        </div>
        <CardContent className="space-y-3 text-xs leading-relaxed !pt-5">
          <div className="rounded-md border bg-card p-3">
            <p className="font-semibold">Native hooks enabled</p>
            <p className="mt-1 text-muted-foreground">
              {providersWithNativeHooks.length > 0
                ? providersWithNativeHooks.join(', ')
                : 'No connected providers with native hook support.'}
            </p>
          </div>

          <div className="rounded-md border bg-card p-3">
            <p className="font-semibold">Assessment</p>
            <p className="mt-1 text-muted-foreground">
              Codex currently has no native hooks surface in config. Ship keeps hook state provider-agnostic, exports to Claude and Gemini, and skips Codex hook export.
            </p>
          </div>

          {providersWithoutNativeHooks.length > 0 && (
            <div className="rounded-md border bg-card p-3">
              <p className="font-semibold">No native hooks</p>
              <p className="mt-1 text-muted-foreground">{providersWithoutNativeHooks.join(', ')}</p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
